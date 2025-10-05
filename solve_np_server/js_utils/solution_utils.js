"use strict";
function maybe_delete(solution_id, solution_title) {
    const dialog = document.getElementById('confirm-dialog');
    const message = document.getElementById('confirm-dialog-message');
    const confirmButton = document.getElementById('confirm-dialog-button');
    const cancelButton = document.getElementById('cancel-dialog-button');
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
function delete_solution(solution_id) {
    fetch(`/api/solutions/${solution_id}`, { method: 'DELETE' })
        .then(() => {
        location.reload();
    })
        .catch(console.log);
}
function new_attempt(button) {
    const table = button.closest('table');
    const new_attempt_row = table.querySelector('#new_attempt_row');
    const create_cancel_row = table.querySelector('#create_cancel_row');
    const create_button = create_cancel_row.querySelector('#create_button');
    const cancel_button = create_cancel_row.querySelector('#cancel_button');
    const key_handler = (event) => {
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
    const title_input = new_row.querySelector("#new_title");
    const description_input = new_row.querySelector("#new_description");
    title_input.focus();
    title_input.addEventListener('keydown', key_handler);
    description_input.addEventListener('keydown', key_handler);
}
function cancel_new_attempt(button) {
    const table = button.closest('table');
    const new_attempt_row = table.querySelector('#new_attempt_row');
    const create_cancel_row = table.querySelector('#create_cancel_row');
    const error_msg = table.querySelector('#create_attempt_error_message');
    table.deleteRow(table.rows.length - 3);
    new_attempt_row.style.display = 'contents';
    create_cancel_row.style.display = 'none';
    error_msg.innerHTML = '';
}
function create_solution(button, problemId) {
    const table = button.closest('table');
    const title = table.querySelector('#new_title');
    const description = table.querySelector('#new_description');
    const error_msg = table.querySelector('#create_attempt_error_message');
    if (title.value.trim() === '') {
        error_msg.innerHTML = '\'Title\' field is mandatory.';
        return;
    }
    fetch('/api/solutions', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ problem_id: problemId, title: title.value, description: description.value }),
    })
        .then(response => {
        if (response.ok) {
            location.reload();
        }
        else {
            return response.text().then(error => Promise.reject(error));
        }
    })
        .catch((error) => {
        error_msg.innerHTML = `${error}`;
    });
}
