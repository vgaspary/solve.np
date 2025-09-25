function maybe_delete(solution_id, solution_title) {
    var dialog = document.getElementById('confirm-dialog');
    var message = document.getElementById('confirm-dialog-message');
    var confirmButton = document.getElementById('confirm-dialog-button');
    var cancelButton = document.getElementById('cancel-dialog-button');
    if (dialog && message && confirmButton && cancelButton) {
        message.textContent = "Are you sure you want to delete the ".concat(solution_title, " solution?");
        dialog.style.display = 'flex';
        confirmButton.onclick = function () {
            delete_solution(solution_id);
            dialog.style.display = 'none';
        };
        cancelButton.onclick = function () {
            dialog.style.display = 'none';
        };
    }
}
function delete_solution(solution_id) {
    fetch("/api/solutions/".concat(solution_id), { method: 'DELETE' })
        .then(function () {
        location.reload();
    })
        .catch(console.log);
}
