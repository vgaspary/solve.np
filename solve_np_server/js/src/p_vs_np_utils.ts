import { global_event_listener_stack } from "./event_listener_stack";

enum PvsNp {
  Eq = 'eq',
  Neq = 'neq',
}

function vote(choice: PvsNp) {
  fetch('/api/vote/' + choice, { method: 'POST' })
    .then(() => window.location.reload())
    .catch(console.log);
}

export function maybe_vote(choice: PvsNp) {
  const dialog = document.getElementById('confirm-dialog') as HTMLElement | null;
  if (!dialog) return;
  const message = document.getElementById('confirm-dialog-message') as HTMLElement | null;
  if (!message) return;
  const confirmButton = document.getElementById('confirm-dialog-button') as HTMLElement | null;
  if (!confirmButton) return;
  const cancelButton = document.getElementById('cancel-dialog-button') as HTMLElement | null;
  if (!cancelButton) return;

  message.innerHTML = `You can <b>only vote once</b>. Are you sure about your choice: ${choice === PvsNp.Eq ? "P == NP" : "P != NP"}?`;
  dialog.style.display = 'flex';

  confirmButton.onclick = () => {
    vote(choice);
    dialog.style.display = 'none';
  };

  cancelButton.onclick = () => {
    dialog.style.display = 'none';
  };

  global_event_listener_stack.push((event: Event): void => {
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
