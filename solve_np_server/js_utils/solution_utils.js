function maybe_delete(problem_id, solution_id, solution_title) {
    var dialog = document.getElementById('confirm-dialog');
    var message = document.getElementById('confirm-dialog-message');
    var confirmButton = document.getElementById('confirm-dialog-button');
    var cancelButton = document.getElementById('cancel-dialog-button');
    if (dialog && message && confirmButton && cancelButton) {
        message.textContent = "Are you sure you want to delete the ".concat(solution_title, " solution?");
        dialog.style.display = 'flex';
        confirmButton.onclick = function () {
            delete_solution(problem_id, solution_id);
            dialog.style.display = 'none';
        };
        cancelButton.onclick = function () {
            dialog.style.display = 'none';
        };
    }
}
function delete_solution(problem_id, solution_id) {
    fetch("/api/solutions/".concat(problem_id, "/").concat(solution_id), { method: 'DELETE' })
        .then(function () {
        location.reload();
    })
        .catch(console.log);
}
function new_attempt(button) {
    var table = button.closest('table');
    var new_attempt_row = table.querySelector('#new_attempt_row');
    var create_cancel_row = table.querySelector('#create_cancel_row');
    var new_row = table.insertRow(table.rows.length - 2);
    new_row.innerHTML = "\n    <td></td>\n    <td></td>\n    <td><input type=\"text\" autocomplete=\"off\" placeholder=\"Title\" id=\"new_title\"/></td>\n    <td><input type=\"text\" autocomplete=\"off\" placeholder=\"Description (optional)\" id=\"new_description\"/></td>\n  ";
    new_attempt_row.style.display = 'none';
    create_cancel_row.style.display = 'contents';
}
function cancel_new_attempt(button) {
    var table = button.closest('table');
    var new_attempt_row = table.querySelector('#new_attempt_row');
    var create_cancel_row = table.querySelector('#create_cancel_row');
    var error_msg = table.querySelector('#create_attempt_error_message');
    table.deleteRow(table.rows.length - 3);
    new_attempt_row.style.display = 'contents';
    create_cancel_row.style.display = 'none';
    error_msg.innerHTML = '';
}
function create_solution(button, problemId) {
    var table = button.closest('table');
    var title = table.querySelector('#new_title');
    var description = table.querySelector('#new_description');
    fetch('/api/solutions', {
        method: 'POST',
        body: JSON.stringify({ problem_id: problemId, title: title.value, description: description.value }),
        headers: {
            'Content-Type': 'application/json',
        },
    })
        .then(function (response) {
        if (response.ok) {
            location.reload();
        }
        else {
            throw response.body;
        }
    })
        .catch(function (error) {
        var error_msg = table.querySelector('#create_attempt_error_message');
        error_msg.innerHTML = "".concat(error);
    });
}
