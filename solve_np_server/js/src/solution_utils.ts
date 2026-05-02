import { global_event_listener_stack } from "./event_listener_stack";
import { parse_solution } from "./problem_utils";

type CancellationToken = { stop: boolean };
type Solution = {
  title: string,
  description: string,
}

function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.appendChild(document.createTextNode(text));
  return div.innerHTML;
}

async function maybe_edit_solution(solution_id: string, problemId: string, titleInput: HTMLInputElement, descriptionInput: HTMLInputElement): Promise<void> {
  if (titleInput.value.trim() === '') {
    throw `'Title' field is mandatory.`;
  }
  const updateResponse = await fetch(
    `/api/solutions/${solution_id}`,
    {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title: titleInput.value.trim(), description: descriptionInput.value.trim() })
    });
  if (!updateResponse.ok) {
    throw await updateResponse.text();
  }
  await reload_solution_table(problemId);
}

async function reload_solution_table(problemId: string): Promise<void> {
  const solutionTable = document.getElementById(`${problemId}_solution_table`) as HTMLElement;
  if (!solutionTable) return;
  const response = await fetch(`/solution_table/${problemId}`);
  const content = await response.text();
  solutionTable.innerHTML = content;
}

export function edit_solution(button: HTMLElement, solutionId: string, problemId: string): void {
  const solutionTable = button.closest('table');
  if (!solutionTable) return;
  const currentRow = button.closest('tr');
  if (!currentRow) return;
  const rowIdx = currentRow.rowIndex;

  (async () => {
    const response: Response = await fetch(`/api/solutions/${solutionId}`);
    if (!response.ok) return;
    const solution: Solution = JSON.parse(await response.text());

    solutionTable.deleteRow(rowIdx);
    const editRow = solutionTable.insertRow(rowIdx);
    editRow.innerHTML = `
    <td></td>
    <td></td>
    <td></td>
    <td><input type="text" autocomplete="off" placeholder="Title" value="${solution.title}" /></td>
    <td><input type="text" autocomplete="off" placeholder="Description (optional)" value="${solution.description}" style="width: 100%;" /></td>
    `;
    const titleInput = editRow.cells[3]?.firstChild! as HTMLInputElement;
    const descriptionInput = editRow.cells[4]?.firstChild! as HTMLInputElement;

    const saveCancelRow = solutionTable.insertRow(rowIdx + 1);
    saveCancelRow.innerHTML = `
    <td colspan="2"><a><i>[Save]</i></a></td>
    <td><a><i>[Cancel]</i></a></td>
    <td colspan="2"><span class="error-message"></span></td>
    `;
    const saveButton = saveCancelRow.cells[0]?.firstChild! as HTMLAnchorElement;
    const cancelButton = saveCancelRow.cells[1]?.firstChild! as HTMLAnchorElement;
    const errorMsgLabel = saveCancelRow.cells[2]?.firstChild! as HTMLSpanElement;

    global_event_listener_stack.push((event: Event): void => {
      if (event instanceof KeyboardEvent) {
        switch (event.key) {
          case 'Enter':
            saveButton.click();
            break;
          case 'Escape':
            cancelButton.click();
            break;
        }
      }
    });

    saveButton.onclick = () => {
      (async () => {
        try {
          await maybe_edit_solution(solutionId, problemId, titleInput, descriptionInput);
          global_event_listener_stack.pop();
        } catch (err) {
          errorMsgLabel.innerHTML = `${err}`;
        }
      })();
    };
    cancelButton.onclick = () => {
      (async () => {
        await reload_solution_table(problemId);
        global_event_listener_stack.pop();
      })();
    };
  })();
}

export function edit_solution_title_desc(solutionId: string, problemId: string): void {
  (async () => {
    try {
      const response: Response = await fetch(`/api/solutions/${solutionId}`);
      if (!response.ok) throw await response.text();
      const solution: Solution = JSON.parse(await response.text());

      const titleDiv = document.getElementById('title');
      if (!titleDiv) return;
      const descriptionDiv = document.getElementById('description');
      if (!descriptionDiv) return;
      titleDiv.innerHTML = `<input type="text" autocomplete="off" placeholder="Title" id="edit_title_input" value="${escapeHtml(solution.title)}" />`;
      descriptionDiv.innerHTML = `<input type="text" autocomplete="off" placeholder="Description (optional)" id="edit_description_input" value="${escapeHtml(solution.description)}" style="width: 90%;" />`;
      const titleInput = document.getElementById('edit_title_input') as HTMLInputElement;
      const descriptionInput = document.getElementById('edit_description_input') as HTMLInputElement;
      const controlsSpan = document.createElement('span');
      controlsSpan.innerHTML = `
        <a id="save_solution_button" style="margin-left: 10px; cursor: pointer;" tabindex="0"><i>[save]</i></a>
        <a id="cancel_solution_button" style="margin-left: 5px; cursor: pointer;" tabindex="0"><i>[cancel]</i></a>
        <span id="edit_solution_error_message" class="error-message" style="margin-left: 10px;"></span>
      `;
      titleDiv.appendChild(controlsSpan);
      const saveButton = document.getElementById('save_solution_button') as HTMLAnchorElement;
      const cancelButton = document.getElementById('cancel_solution_button') as HTMLAnchorElement;
      const errorMessageSpan = document.getElementById('edit_solution_error_message') as HTMLSpanElement;
      titleInput.focus();

      global_event_listener_stack.push((event: Event): void => {
        if (event instanceof KeyboardEvent) {
          switch (event.key) {
            case 'Enter':
              if (document.activeElement === cancelButton) {
                cancelButton.click();
              } else {
                saveButton.click();
              }
              break;
            case 'Escape':
              cancelButton.click();
              break;
          }
        }
      });

      saveButton.onclick = async () => {
        try {
          await maybe_edit_solution(solutionId, problemId, titleInput, descriptionInput);
          global_event_listener_stack.pop();
          window.location.reload();
        } catch (err) {
          errorMessageSpan.innerHTML = `${err}`;
        }
      };

      cancelButton.onclick = () => {
        global_event_listener_stack.pop();
        window.location.reload();
      };

    } catch (error) {
      console.error('Error in edit_solution_title_desc:', error);
      // Display error if initial fetch fails or other critical errors
      const titleDiv = document.getElementById('title');
      if (titleDiv && titleDiv.parentNode) {
        const errorDisplay = document.createElement('span');
        errorDisplay.className = 'error-message';
        errorDisplay.innerHTML = `Failed to load solution for editing: ${error}`;
        titleDiv.parentNode.insertBefore(errorDisplay, titleDiv.nextSibling);
      }
    }
  })();
}

export function maybe_delete(solution_id: string, problem_id: string, input_id: string, solution_title: string): void {
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
    delete_solution(solution_id, problem_id, input_id, solution_table);
    dialog.style.display = 'none';
  };

  cancel_button.onclick = () => {
    global_event_listener_stack.pop();
    dialog.style.display = 'none';
  };
}

function delete_solution(solution_id: string, problem_id: string, input_id: string, solution_table: HTMLElement): void {
  fetch(`/api/solutions/${solution_id}`, { method: 'DELETE' })
    .then(() => load_solution_table(problem_id, input_id))
    .then(content => solution_table.innerHTML = content)
    .catch(console.log);
}

async function load_solution_table(problem_id: string, input_id: string | null): Promise<string> {
  const query_params = input_id ? `?input_id=${input_id}` : "";
  const response = await fetch(`/solution_table/${problem_id}${query_params}`);
  return await response.text();
}

export function new_attempt(button: HTMLElement): void {
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
    <td></td>
    <td><input type="text" autocomplete="off" placeholder="Title" id="new_title" /></td>
    <td><input type="text" autocomplete="off" placeholder="Description (optional)" id="new_description" style="width: 100%;" /></td>
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

export function cancel_new_attempt(button: HTMLElement): void {
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

export function create_solution(button: HTMLElement, problemId: string): void {
  const table = button.closest('table')!;
  const title = table.querySelector('#new_title') as HTMLInputElement;
  const description = table.querySelector('#new_description') as HTMLInputElement;
  const error_msg = table.querySelector('#create_attempt_error_message') as HTMLElement;

  if (title.value.trim() === '') {
    error_msg.innerHTML = '\'Title\' field is mandatory.';
    global_event_listener_stack.pop();
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

      await reload_solution_table(problemId);
      global_event_listener_stack.pop();
    } catch (err) {
      error_msg.innerHTML = `${err}`;
      global_event_listener_stack.pop();
    }
  })();
}

export function submit_solution(problem_id: string, solution_id: string, input_id: string): void {
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
      if (response.status === 500) {
        const errorHtml = await response.text();
        const newHtml = errorHtml.replace('<head>', `<head><base href="${window.location.origin}">`);
        const blob = new Blob([newHtml], { type: 'text/html' });
        window.location.assign(URL.createObjectURL(blob));
        return;
      }
      if (response.status !== 200) {
        throw new Error("Malformed solution, failed parsing.");
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
    } catch (err) {
      solution_textarea.classList = "error";
      if (err instanceof Error) {
        status_paragraph.innerHTML = err.message;
      } else {
        status_paragraph.innerHTML = "Malformed solution, failed parsing.";
      }
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

export function choose_solution(problem_id: string, input_id: string) {
  (async () => {
    let content = await load_solution_table(problem_id, input_id);
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

export function maybe_expand() {
  const queryParams = new URLSearchParams(window.location.search);
  const input = queryParams.get('input_id');
  if (!input) {
    return;
  }
  const details = document.getElementById(input) as HTMLDetailsElement | null;
  if (!details) {
    return;
  }
  details.open = true;
}
