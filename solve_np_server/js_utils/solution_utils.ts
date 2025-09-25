function maybe_delete(solution_id: string, solution_title: string): void {
  const dialog = document.getElementById('confirm-dialog') as HTMLElement;
  const message = document.getElementById('confirm-dialog-message') as HTMLElement;
  const confirmButton = document.getElementById('confirm-dialog-button') as HTMLElement;
  const cancelButton = document.getElementById('cancel-dialog-button') as HTMLElement;

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

function delete_solution(solution_id: string): void {
  fetch(`/api/solutions/${solution_id}`, { method: 'DELETE' })
    .then(() => {
      location.reload();
    })
    .catch(console.log);
}
