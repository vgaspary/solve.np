"use strict";
var PvsNp;
(function (PvsNp) {
    PvsNp["Eq"] = "eq";
    PvsNp["Neq"] = "neq";
})(PvsNp || (PvsNp = {}));
function vote(choice) {
    fetch('/api/vote/' + choice, { method: 'POST' })
        .then(() => window.location.reload())
        .catch(console.log);
}
function maybe_vote(choice) {
    const dialog = document.getElementById('confirm-dialog');
    const message = document.getElementById('confirm-dialog-message');
    const confirmButton = document.getElementById('confirm-dialog-button');
    const cancelButton = document.getElementById('cancel-dialog-button');
    if (dialog && message && confirmButton && cancelButton) {
        message.innerHTML = `You can <b>only vote once</b>. Are you sure about your choice: ${choice === PvsNp.Eq ? "P == NP" : "P != NP"}?`;
        dialog.style.display = 'flex';
        confirmButton.onclick = () => {
            vote(choice);
            dialog.style.display = 'none';
        };
        cancelButton.onclick = () => {
            dialog.style.display = 'none';
        };
    }
}
