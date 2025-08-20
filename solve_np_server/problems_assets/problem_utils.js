function md_to_html(md_text) {
  return md_text.replace(/(\*\*|\*|__|_)(.*?)(\*\*|\*|__|_)/g, '<em>$2</em>');
}

function render_problem(problem) {
  return `
        <li>
          <h1><a href="/problems/${problem.id}">${problem.name}</a></h1>
          <article><p>${md_to_html(problem.description)}</p></article>
        </li>
          `;
}

function parse_solution(problem_id, input) {
  if (problem_id == "travelling_salesman") {
    return { TspSolution: input.trim().split(/\s+/).map(n => Number(n)) };
  } else if (problem_id == "graph_coloring") {
    return { GraphColoringSolution: input.trim().split(/\s+/).map(n => Number(n)) };
  }
}

function check_solution(problem_id) {
  let solution_textarea = document.getElementById("sample_solution");
  let solution = solution_textarea.value;
  let sample_solution_result = document.getElementById('sample_solution_result');
  if (solution === "") {
    solution_textarea.classList = [""];
    sample_solution_result.innerHTML = "";
    return;
  }

  fetch('/api/validate_sample_solution',
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(parse_solution(problem_id, solution))
    })
    .then(response => response.json())
    .then(validation_result => {
      if (validation_result.Valid) {
        solution_textarea.classList = ["success"];
        sample_solution_result.innerHTML = validation_result.Valid;
      } else if (validation_result.Invalid) {
        solution_textarea.classList = ["error"];
        sample_solution_result.innerHTML = validation_result.Invalid;
      }
    })
    .catch(console.log);
}
