var PvsNp;
(function (PvsNp) {
    PvsNp["Eq"] = "eq";
    PvsNp["Neq"] = "neq";
})(PvsNp || (PvsNp = {}));
function vote(choice) {
    fetch('/api/vote/' + choice, { method: 'POST' })
        .then(function () { return window.location.reload(); })
        .catch(console.log);
}
function maybe_vote(choice) {
    var dialog = document.getElementById('confirm-dialog');
    var message = document.getElementById('confirm-dialog-message');
    var confirmButton = document.getElementById('confirm-dialog-button');
    var cancelButton = document.getElementById('cancel-dialog-button');
    if (dialog && message && confirmButton && cancelButton) {
        message.innerHTML = "You can <b>only vote once</b>. Are you sure about your choice: ".concat(choice === PvsNp.Eq ? "P == NP" : "P != NP", "?");
        dialog.style.display = 'flex';
        confirmButton.onclick = function () {
            vote(choice);
            dialog.style.display = 'none';
        };
        cancelButton.onclick = function () {
            dialog.style.display = 'none';
        };
    }
}
