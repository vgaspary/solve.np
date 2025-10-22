type CancellationToken = { stop: boolean };

class EventListenerStack {
  private readonly target: EventTarget;
  private readonly type: string;
  private event_listener_stack: EventListenerOrEventListenerObject[] = [];

  constructor(target: EventTarget, type: string) {
    this.target = target;
    this.type = type;
  }

  public push(event_listener: EventListenerOrEventListenerObject): void {
    const current_listener = this.event_listener_stack.at(-1);
    if (current_listener) {
      this.target.removeEventListener(this.type, current_listener);
    }

    this.event_listener_stack.push(event_listener);
    this.target.addEventListener(this.type, event_listener);
  }

  public pop(): void {
    const current_listener = this.event_listener_stack.pop();
    if (current_listener) {
      this.target.removeEventListener(this.type, current_listener);

      const previous_listener = this.event_listener_stack.at(-1);
      if (previous_listener) {
        this.target.addEventListener(this.type, previous_listener);
      }
    }
  }
}

const global_event_listener_stack = new EventListenerStack(window, 'keydown');

function maybe_delete(solution_id: string, problem_id: string, solution_title: string): void {
  const dialog = document.getElementById('confirm-dialog');
  if (!dialog) return;
  const message = document.getElementById('confirm-dialog-message');
  if (!message) return;
  const confirm_button = document.getElementById('confirm-dialog-button');
  if (!confirm_button) return;
  const cancel_button = document.getElementById('cancel-dialog-button');
  if (!cancel_button) return;
  const solution_table = document.getElementById(`${problem_id}_solution_table`);
  if (!solution_table) return;

  global_event_listener_stack.push((event: Event): void => {
    if (event instanceof KeyboardEvent) {
      switch (event.key) {
        case 'Enter':
          confirm_button.click();
          break;
        case 'Escape':
          cancel_button.click();
          break;
      }
    }
  });

  message.textContent = `Are you sure you want to delete the ${solution_title} solution?`;
  dialog.style.display = 'flex';

  confirm_button.onclick = () => {
    global_event_listener_stack.pop();
    delete_solution(solution_id, problem_id, solution_table);
    dialog.style.display = 'none';
  };

  cancel_button.onclick = () => {
    global_event_listener_stack.pop();
    dialog.style.display = 'none';
  };
}

function delete_solution(solution_id: string, problem_id: string, solution_table: HTMLElement): void {
  fetch(`/api/solutions/${solution_id}`, { method: 'DELETE' })
    .then(() => load_solution_table(problem_id))
    .then(content => solution_table.innerHTML = content)
    .catch(console.log);
}

async function load_solution_table(problem_id: string): Promise<string> {
  const response = await fetch(`/solution_table/${problem_id}`);
  return await response.text();
}

function new_attempt(button: HTMLElement): void {
  const table = button.closest('table')!;
  const new_attempt_row = table.querySelector('#new_attempt_row') as HTMLElement;
  const create_cancel_row = table.querySelector('#create_cancel_row') as HTMLElement;
  const create_button = create_cancel_row.querySelector('#create_button') as HTMLElement;
  const cancel_button = create_cancel_row.querySelector('#cancel_button') as HTMLElement;

  const new_row = table.insertRow(table.rows.length - 2);
  new_attempt_row.style.display = 'none';
  create_cancel_row.style.display = 'contents';
  new_row.innerHTML = `
    <td></td>
    <td></td>
    <td><input type="text" autocomplete="off" placeholder="Title" id="new_title"/></td>
    <td><input type="text" autocomplete="off" placeholder="Description (optional)" id="new_description"/></td>
  `;

  const title_input = new_row.querySelector("#new_title") as HTMLElement;
  title_input.focus();
  global_event_listener_stack.push((event: Event): void => {
    if (event instanceof KeyboardEvent) {
      switch (event.key) {
        case 'Enter':
          create_button.click();
          break;
        case 'Escape':
          cancel_button.click();
          break;
      }
    }
  });
}

function cancel_new_attempt(button: HTMLElement): void {
  const table = button.closest('table')!;
  const new_attempt_row = table.querySelector('#new_attempt_row') as HTMLElement;
  const create_cancel_row = table.querySelector('#create_cancel_row') as HTMLElement;
  const error_msg = table.querySelector('#create_attempt_error_message') as HTMLElement;

  table.deleteRow(table.rows.length - 3);
  new_attempt_row.style.display = 'contents';
  create_cancel_row.style.display = 'none';
  error_msg.innerHTML = '';
  global_event_listener_stack.pop();
}

function create_solution(button: HTMLElement, problemId: string): void {
  const table = button.closest('table')!;
  const title = table.querySelector('#new_title') as HTMLInputElement;
  const description = table.querySelector('#new_description') as HTMLInputElement;
  const error_msg = table.querySelector('#create_attempt_error_message') as HTMLElement;
  const solution_table = document.getElementById(`${problemId}_solution_table`) as HTMLElement;

  if (title.value.trim() === '') {
    error_msg.innerHTML = '\'Title\' field is mandatory.';
    return;
  }

  (async () => {
    try {
      const create_response = await fetch('/api/solutions',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ problem_id: problemId, title: title.value, description: description.value }),
        });
      if (!create_response.ok) {
        throw await create_response.text();
      }

      const response = await fetch(`/solution_table/${problemId}`);
      const content = await response.text();
      solution_table.innerHTML = content;
      global_event_listener_stack.pop();
    } catch (err) {
      error_msg.innerHTML = `${err}`;
    }
  })();
}

function submit_solution(problem_id: string, solution_id: string, input_id: string): void {
  const solution_textarea = document.getElementById(input_id + '_solution') as HTMLTextAreaElement | null;
  if (!solution_textarea) return;
  solution_textarea.classList = "";
  const solution = solution_textarea.value;
  if (solution.trim() === '') {
    return;
  }

  const status_paragraph = document.getElementById(input_id + '_status') as HTMLParagraphElement | null;
  if (!status_paragraph) return;
  const status_label = document.getElementById(input_id + '_status_label') as HTMLSpanElement | null;
  if (!status_label) return;

  const cancellation_token: CancellationToken = { stop: false };
  show_in_progress(status_paragraph, 0, cancellation_token);

  (async () => {
    try {
      const response = await
        fetch(`/api/submit_solution/${problem_id}/${solution_id}/${input_id}`,
          {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json'
            },
            body: JSON.stringify(parse_solution(problem_id, solution))
          });
      cancellation_token.stop = true;
      if (response.status === 204) {
        status_paragraph.innerHTML = "This solution has already been submitted.";
        solution_textarea.classList = "";
        return;
      }
      const validation_result = await response.json();
      if (validation_result.Valid) {
        status_paragraph.innerHTML = 'Score: ' + validation_result.Valid;
        solution_textarea.classList = "success";
      } else if (validation_result.Invalid) {
        status_paragraph.innerHTML = validation_result.Invalid;
        solution_textarea.classList = "error";
      }
      await maybe_update_status(solution_id, input_id);
    } finally {
      cancellation_token.stop = true;
    }
  })();
}

async function maybe_update_status(solution_id: string, input_id: string): Promise<void> {
  const submission_status = document.getElementById(`${input_id}_status_label`);
  if (submission_status) {
    await maybe_update_status_helper(`../api/solutions/${solution_id}/input/${input_id}/status`, submission_status);
  }

  const solution_status = document.getElementById('solution_status');
  if (solution_status) {
    await maybe_update_status_helper(`../api/solutions/${solution_id}/status`, solution_status);
  }
}

async function maybe_update_status_helper(resource: string, target: HTMLElement): Promise<void> {
  const response = await fetch(resource);
  const status = await response.json();
  if (status === 'Untested') {
    target.innerHTML = 'Untested';
    target.style.color = 'grey';
  } else if (status === 'Attempted') {
    target.innerHTML = 'Attempted';
    target.style.color = 'orange';
  } else if ('Solved' in status) {
    target.innerHTML = 'Solved';
    target.style.color = 'green';
  }
}

function show_in_progress(paragraph: HTMLParagraphElement, i: number, cancellation_token: CancellationToken) {
  if (cancellation_token.stop) {
    return;
  }
  const progress = '⠋⠙⠸⢰⣠⣄⡆⠇';
  paragraph.innerHTML = progress[i % progress.length] + ' Validation in progress...';
  setTimeout(() => show_in_progress(paragraph, i + 1, cancellation_token), 100);
}

function choose_solution(problem_id: string, input_id: string) {
  (async () => {
    let content = await load_solution_table(problem_id);
    const solution_table = document.getElementById(`${problem_id}_solution_table`) as HTMLElement;
    solution_table.innerHTML = content;

    const dialog = document.getElementById('choose_solution_dialog') as HTMLElement | null;
    if (!dialog) {
      return;
    }
    const template_inner_HTML = dialog.innerHTML;
    dialog.innerHTML = template_inner_HTML.replaceAll('INPUT_ID_PLACEHOLDER', input_id);

    const cancel_button = document.getElementById('cancel_choosing_solution_button') as HTMLButtonElement | null;
    if (!cancel_button) {
      return;
    }

    dialog.style.display = 'flex';
    global_event_listener_stack.push((event: Event): void => {
      if (event instanceof KeyboardEvent && event.key === 'Escape') {
        cancel_button.click();
      }
    });
    cancel_button.onclick = () => {
      dialog.style.display = 'none';
      dialog.innerHTML = template_inner_HTML;
      global_event_listener_stack.pop();
    };
  })();
}
