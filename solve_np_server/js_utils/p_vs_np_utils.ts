enum PvsNp {
  Eq,
  Neq,
}

function vote(choice: PvsNp) {
  fetch('/api/vote/' + choice, { method: 'POST' })
  .then(() => window.location.reload())
  .catch(console.log);
}
