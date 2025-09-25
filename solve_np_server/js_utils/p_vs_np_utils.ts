enum PvsNp {
  Eq = 'eq',
  Neq = 'neq',
}

function vote(choice: PvsNp) {
  fetch('/api/vote/' + choice, { method: 'POST' })
  .then(() => window.location.reload())
  .catch(console.log);
}

function maybe_vote(choice: PvsNp) {
  const dialog = document.getElementById('confirm-dialog') as HTMLElement;
  const message = document.getElementById('confirm-dialog-message') as HTMLElement;
  const confirmButton = document.getElementById('confirm-dialog-button') as HTMLElement;
  const cancelButton = document.getElementById('cancel-dialog-button') as HTMLElement;

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
