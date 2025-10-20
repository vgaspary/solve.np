"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
function maybe_delete(solution_id, problem_id, solution_title) {
    const dialog = document.getElementById('confirm-dialog');
    const message = document.getElementById('confirm-dialog-message');
    const confirmButton = document.getElementById('confirm-dialog-button');
    const cancelButton = document.getElementById('cancel-dialog-button');
    const solution_table = document.getElementById(`${problem_id}_solution_table`);
    if (dialog && message && confirmButton && cancelButton) {
        const key_handler = (event) => {
            switch (event.key) {
                case 'Enter':
                    confirmButton.click();
                    break;
                case 'Escape':
                    cancelButton.click();
                    break;
            }
        };
        window.addEventListener('keydown', key_handler);
        message.textContent = `Are you sure you want to delete the ${solution_title} solution?`;
        dialog.style.display = 'flex';
        confirmButton.onclick = () => {
            window.removeEventListener('keydown', key_handler);
            delete_solution(solution_id, problem_id, solution_table);
            dialog.style.display = 'none';
        };
        cancelButton.onclick = () => {
            window.removeEventListener('keydown', key_handler);
            dialog.style.display = 'none';
        };
    }
}
function delete_solution(solution_id, problem_id, solution_table) {
    fetch(`/api/solutions/${solution_id}`, { method: 'DELETE' })
        .then(() => load_solution_table(problem_id))
        .then(content => solution_table.innerHTML = content)
        .catch(console.log);
}
function load_solution_table(problem_id) {
    return __awaiter(this, void 0, void 0, function* () {
        const response = yield fetch(`/solution_table/${problem_id}`);
        return yield response.text();
    });
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
                break;
            case 'Escape':
                cancel_button.click();
                break;
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
    const solution_table = document.getElementById(`${problemId}_solution_table`);
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
            fetch(`/solution_table/${problemId}`)
                .then(response => response.text())
                .then(content => solution_table.innerHTML = content)
                .catch(console.log);
        }
        else {
            return response.text().then(error => Promise.reject(error));
        }
    })
        .catch((error) => {
        error_msg.innerHTML = `${error}`;
    });
}
function submit_solution(problem_id, solution_id, input_id) {
    const solution_textarea = document.getElementById(input_id + '_solution');
    if (!solution_textarea) {
        throw new Error(input_id + '_solution textarea not found.');
    }
    solution_textarea.classList = "";
    const solution = solution_textarea.value;
    if (solution.trim() === '') {
        return;
    }
    const status_paragraph = document.getElementById(input_id + '_status');
    if (!status_paragraph) {
        throw new Error(input_id + '_status paragraph not found.');
    }
    const status_label = document.getElementById(input_id + '_status_label');
    if (!status_label) {
        throw new Error(input_id + '_status_label span not found.');
    }
    const cancellation_token = { stop: false };
    show_in_progress(status_paragraph, 0, cancellation_token);
    fetch('/api/submit_solution/' + problem_id + '/' + solution_id + '/' + input_id, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(parse_solution(problem_id, solution))
    })
        .then(response => {
        cancellation_token.stop = true;
        if (response.status === 204) {
            status_paragraph.innerHTML = "This solution has already been submitted.";
            solution_textarea.classList = "";
            return;
        }
        return response.json();
    })
        .then(validation_result => {
        if (validation_result.Valid) {
            status_paragraph.innerHTML = 'Score: ' + validation_result.Valid;
            solution_textarea.classList = "success";
            status_label.innerHTML = 'Solved';
            status_label.style = 'color: green; width: 130px; display: inline-block;';
        }
        else if (validation_result.Invalid) {
            status_paragraph.innerHTML = validation_result.Invalid;
            solution_textarea.classList = "error";
            status_label.innerHTML = 'Attempted';
            status_label.style = 'color: orange; width: 130px; display: inline-block;';
        }
    })
        .catch(err => {
        cancellation_token.stop = true;
        console.log(err);
    });
}
function show_in_progress(paragraph, i, cancellation_token) {
    if (cancellation_token.stop) {
        return;
    }
    const progress = '⠋⠙⠸⢰⣠⣄⡆⠇';
    paragraph.innerHTML = progress[i % progress.length] + ' Validation in progress...';
    setTimeout(() => show_in_progress(paragraph, i + 1, cancellation_token), 100);
}
function choose_solution(problem_id, input_id) {
    (() => __awaiter(this, void 0, void 0, function* () {
        let content = yield load_solution_table(problem_id);
        const solution_table = document.getElementById(`${problem_id}_solution_table`);
        solution_table.innerHTML = content;
        const dialog = document.getElementById('choose_solution_dialog');
        if (!dialog) {
            return;
        }
        const template_inner_HTML = dialog.innerHTML;
        dialog.innerHTML = template_inner_HTML.replaceAll('INPUT_ID_PLACEHOLDER', input_id);
        const cancel_button = document.getElementById('cancel_choosing_solution_button');
        if (!cancel_button) {
            return;
        }
        const key_handler = (event) => {
            if (event.key === 'Escape') {
                cancel_button.click();
            }
        };
        dialog.style.display = 'flex';
        window.addEventListener('keydown', key_handler);
        cancel_button.onclick = () => {
            dialog.style.display = 'none';
            dialog.innerHTML = template_inner_HTML;
            window.removeEventListener('keydown', key_handler);
        };
    }))();
}
