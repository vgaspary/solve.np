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
class EventListenerStack {
    constructor(target, type) {
        this.event_listener_stack = [];
        this.target = target;
        this.type = type;
    }
    push(event_listener) {
        const current_listener = this.event_listener_stack.at(-1);
        if (current_listener) {
            this.target.removeEventListener(this.type, current_listener);
        }
        this.event_listener_stack.push(event_listener);
        this.target.addEventListener(this.type, event_listener);
    }
    pop() {
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
function maybe_delete(solution_id, problem_id, input_id, solution_title) {
    const dialog = document.getElementById('confirm-dialog');
    if (!dialog)
        return;
    const message = document.getElementById('confirm-dialog-message');
    if (!message)
        return;
    const confirm_button = document.getElementById('confirm-dialog-button');
    if (!confirm_button)
        return;
    const cancel_button = document.getElementById('cancel-dialog-button');
    if (!cancel_button)
        return;
    const solution_table = document.getElementById(`${problem_id}_solution_table`);
    if (!solution_table)
        return;
    global_event_listener_stack.push((event) => {
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
function delete_solution(solution_id, problem_id, input_id, solution_table) {
    fetch(`/api/solutions/${solution_id}`, { method: 'DELETE' })
        .then(() => load_solution_table(problem_id, input_id))
        .then(content => solution_table.innerHTML = content)
        .catch(console.log);
}
function load_solution_table(problem_id, input_id) {
    return __awaiter(this, void 0, void 0, function* () {
        const query_params = input_id ? `?input_id=${input_id}` : "";
        const response = yield fetch(`/solution_table/${problem_id}${query_params}`);
        return yield response.text();
    });
}
function new_attempt(button) {
    const table = button.closest('table');
    const new_attempt_row = table.querySelector('#new_attempt_row');
    const create_cancel_row = table.querySelector('#create_cancel_row');
    const create_button = create_cancel_row.querySelector('#create_button');
    const cancel_button = create_cancel_row.querySelector('#cancel_button');
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
    title_input.focus();
    global_event_listener_stack.push((event) => {
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
function cancel_new_attempt(button) {
    const table = button.closest('table');
    const new_attempt_row = table.querySelector('#new_attempt_row');
    const create_cancel_row = table.querySelector('#create_cancel_row');
    const error_msg = table.querySelector('#create_attempt_error_message');
    table.deleteRow(table.rows.length - 3);
    new_attempt_row.style.display = 'contents';
    create_cancel_row.style.display = 'none';
    error_msg.innerHTML = '';
    global_event_listener_stack.pop();
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
    (() => __awaiter(this, void 0, void 0, function* () {
        try {
            const create_response = yield fetch('/api/solutions', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ problem_id: problemId, title: title.value, description: description.value }),
            });
            if (!create_response.ok) {
                throw yield create_response.text();
            }
            const response = yield fetch(`/solution_table/${problemId}`);
            const content = yield response.text();
            solution_table.innerHTML = content;
            global_event_listener_stack.pop();
        }
        catch (err) {
            error_msg.innerHTML = `${err}`;
        }
    }))();
}
function submit_solution(problem_id, solution_id, input_id) {
    const solution_textarea = document.getElementById(input_id + '_solution');
    if (!solution_textarea)
        return;
    solution_textarea.classList = "";
    const solution = solution_textarea.value;
    if (solution.trim() === '') {
        return;
    }
    const status_paragraph = document.getElementById(input_id + '_status');
    if (!status_paragraph)
        return;
    const status_label = document.getElementById(input_id + '_status_label');
    if (!status_label)
        return;
    const cancellation_token = { stop: false };
    show_in_progress(status_paragraph, 0, cancellation_token);
    (() => __awaiter(this, void 0, void 0, function* () {
        try {
            const response = yield fetch(`/api/submit_solution/${problem_id}/${solution_id}/${input_id}`, {
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
            const validation_result = yield response.json();
            if (validation_result.Valid) {
                status_paragraph.innerHTML = 'Score: ' + validation_result.Valid;
                solution_textarea.classList = "success";
            }
            else if (validation_result.Invalid) {
                status_paragraph.innerHTML = validation_result.Invalid;
                solution_textarea.classList = "error";
            }
            yield maybe_update_status(solution_id, input_id);
        }
        finally {
            cancellation_token.stop = true;
        }
    }))();
}
function maybe_update_status(solution_id, input_id) {
    return __awaiter(this, void 0, void 0, function* () {
        const submission_status = document.getElementById(`${input_id}_status_label`);
        if (submission_status) {
            yield maybe_update_status_helper(`../api/solutions/${solution_id}/input/${input_id}/status`, submission_status);
        }
        const solution_status = document.getElementById('solution_status');
        if (solution_status) {
            yield maybe_update_status_helper(`../api/solutions/${solution_id}/status`, solution_status);
        }
    });
}
function maybe_update_status_helper(resource, target) {
    return __awaiter(this, void 0, void 0, function* () {
        const response = yield fetch(resource);
        const status = yield response.json();
        if (status === 'Untested') {
            target.innerHTML = 'Untested';
            target.style.color = 'grey';
        }
        else if (status === 'Attempted') {
            target.innerHTML = 'Attempted';
            target.style.color = 'orange';
        }
        else if ('Solved' in status) {
            target.innerHTML = 'Solved';
            target.style.color = 'green';
        }
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
        let content = yield load_solution_table(problem_id, input_id);
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
        dialog.style.display = 'flex';
        global_event_listener_stack.push((event) => {
            if (event instanceof KeyboardEvent && event.key === 'Escape') {
                cancel_button.click();
            }
        });
        cancel_button.onclick = () => {
            dialog.style.display = 'none';
            dialog.innerHTML = template_inner_HTML;
            global_event_listener_stack.pop();
        };
    }))();
}
