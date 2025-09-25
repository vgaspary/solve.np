function maybe_delete(solution_id, solution_title) {
    if (confirm("Are you sure you want to delete the ".concat(solution_title, " solution?"))) {
        fetch("/api/solutions/".concat(solution_id), {
            method: 'DELETE',
        })
            .catch(console.log);
    }
}
