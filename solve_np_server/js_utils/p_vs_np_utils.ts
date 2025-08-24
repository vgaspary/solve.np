enum PvsNp {
  Eq,
  Neq,
}

function vote(choice: PvsNp) {
  fetch('/api/vote/' + choice, { method: 'POST' })
  .catch(console.log);
}
