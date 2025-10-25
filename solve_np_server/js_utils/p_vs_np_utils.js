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
    if (!dialog)
        return;
    const message = document.getElementById('confirm-dialog-message');
    if (!message)
        return;
    const confirmButton = document.getElementById('confirm-dialog-button');
    if (!confirmButton)
        return;
    const cancelButton = document.getElementById('cancel-dialog-button');
    if (!cancelButton)
        return;
    message.innerHTML = `You can <b>only vote once</b>. Are you sure about your choice: ${choice === PvsNp.Eq ? "P == NP" : "P != NP"}?`;
    dialog.style.display = 'flex';
    confirmButton.onclick = () => {
        vote(choice);
        dialog.style.display = 'none';
    };
    cancelButton.onclick = () => {
        dialog.style.display = 'none';
    };
    global_event_listener_stack.push((event) => {
        if (event instanceof KeyboardEvent) {
            switch (event.key) {
                case 'Enter':
                    confirmButton.click();
                    break;
                case 'Escape':
                    cancelButton.click();
                    break;
            }
        }
    });
}
