function maybe_delete(problem_id: string, solution_id: string, solution_title: string): void {
  const dialog = document.getElementById('confirm-dialog') as HTMLElement;
  const message = document.getElementById('confirm-dialog-message') as HTMLElement;
  const confirmButton = document.getElementById('confirm-dialog-button') as HTMLElement;
  const cancelButton = document.getElementById('cancel-dialog-button') as HTMLElement;

  if (dialog && message && confirmButton && cancelButton) {
    message.textContent = `Are you sure you want to delete the ${solution_title} solution?`;
    dialog.style.display = 'flex';

    confirmButton.onclick = () => {
      delete_solution(problem_id, solution_id);
      dialog.style.display = 'none';
    };

    cancelButton.onclick = () => {
      dialog.style.display = 'none';
    };
  }
}

function delete_solution(problem_id: string, solution_id: string): void {
  fetch(`/api/solutions/${problem_id}/${solution_id}`, { method: 'DELETE' })
    .then(() => {
      location.reload();
    })
    .catch(console.log);
}

function new_attempt(button: HTMLElement): void {
  const table = button.closest('table')!;
  const new_attempt_row = table.querySelector('#new_attempt_row') as HTMLElement;
  const create_cancel_row = table.querySelector('#create_cancel_row') as HTMLElement;
  const new_row = table.insertRow(table.rows.length - 2);

  new_row.innerHTML = `
    <td></td>
    <td></td>
    <td><input type="text" autocomplete="off" placeholder="Title" id="new_title"/></td>
    <td><input type="text" autocomplete="off" placeholder="Description (optional)" id="new_description"/></td>
  `;
  new_attempt_row.style.display = 'none';
  create_cancel_row.style.display = 'contents';
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

  fetch('/api/solutions', {
    method: 'POST',
    body: JSON.stringify({ problem_id: problemId, title: title.value, description: description.value }),
    headers: {
      'Content-Type': 'application/json',
    },
  })
    .then((response) => {
      if (response.ok) {
        location.reload();
      } else {
        throw response.body;
      }
    })
    .catch((error) => {
      const error_msg = table.querySelector('#create_attempt_error_message') as HTMLElement;
      error_msg.innerHTML = `${error}`;
    });
}

