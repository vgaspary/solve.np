function maybe_delete(solution_id: string, solution_title: string): void {
  const dialog = document.getElementById('confirm-dialog') as HTMLElement;
  const message = document.getElementById('confirm-dialog-message') as HTMLElement;
  const confirmButton = document.getElementById('confirm-dialog-button') as HTMLElement;
  const cancelButton = document.getElementById('cancel-dialog-button') as HTMLElement;

  if (dialog && message && confirmButton && cancelButton) {
    message.textContent = `Are you sure you want to delete the ${solution_title} solution?`;
    dialog.style.display = 'flex';

    confirmButton.onclick = () => {
      delete_solution(solution_id);
      dialog.style.display = 'none';
    };

    cancelButton.onclick = () => {
      dialog.style.display = 'none';
    };
  }
}

function delete_solution(solution_id: string): void {
  fetch(`/api/solutions/${solution_id}`, { method: 'DELETE' })
    .then(() => {
      location.reload();
    })
    .catch(console.log);
}

function new_attempt(button: HTMLElement): void {
  const table = button.closest('table')!;
  const new_attempt_row = table.querySelector('#new_attempt_row') as HTMLElement;
  const create_cancel_row = table.querySelector('#create_cancel_row') as HTMLElement;
  const create_button = create_cancel_row.querySelector('#create_button') as HTMLElement;
  const cancel_button = create_cancel_row.querySelector('#cancel_button') as HTMLElement;
  const key_handler = (event: KeyboardEvent): void => {
    switch (event.key) {
      case 'Enter':
        create_button.click();
      case 'Escape':
        cancel_button.click();
    }
  };

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
  const description_input = new_row.querySelector("#new_description") as HTMLElement;
  title_input.focus();
  title_input.addEventListener('keydown', key_handler);
  description_input.addEventListener('keydown', key_handler);
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
}

function create_solution(button: HTMLElement, problemId: string): void {
  const table = button.closest('table')!;
  const title = table.querySelector('#new_title') as HTMLInputElement;
  const description = table.querySelector('#new_description') as HTMLInputElement;
  const error_msg = table.querySelector('#create_attempt_error_message') as HTMLElement;

  if (title.value.trim() === '') {
    error_msg.innerHTML = '\'Title\' field is mandatory.';
    return;
  }

  fetch('/api/solutions',
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ problem_id: problemId, title: title.value, description: description.value }),
    })
    .then(response => {
      if (response.ok) {
        location.reload();
      } else {
        return response.text().then(error => Promise.reject(error));
      }
    })
    .catch((error) => {
      error_msg.innerHTML = `${error}`;
    });
}

