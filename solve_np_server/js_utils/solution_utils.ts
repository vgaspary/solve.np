function maybe_delete(solution_id: string, solution_title: string) {
  if (confirm(`Are you sure you want to delete the ${solution_title} solution?`)) {
    fetch(`/api/solutions/${solution_id}`,
      {
        method: 'DELETE',
      })
      .catch(console.log);
  }
}
