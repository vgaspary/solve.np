type SampleSolution = { TspSolution: number[] } | { GraphColoringSolution: number[] } | { VertexCoverSolution: number[] };

export function parse_solution(problem_id: string, input: string): SampleSolution | null {
  switch (problem_id) {
    case 'travelling_salesman':
      return { TspSolution: input.trim().split(/\s+/).map(n => Number(n)) };
    case 'graph_coloring':
      return { GraphColoringSolution: input.trim().split(/\s+/).map(n => Number(n)) };
    case 'vertex_cover':
      return { VertexCoverSolution: input.trim().split(/\s+/).map(n => Number(n)) };
    default:
      return null;
  }
}

export function check_solution(problem_id: string): void {
  const solution_textarea = document.getElementById("sample_solution") as HTMLTextAreaElement | null;
  if (!solution_textarea) {
    throw new Error("'sample_solution' textarea not found.");
  }
  const solution = solution_textarea.value;
  const sample_solution_result = document.getElementById('sample_solution_result');
  if (!sample_solution_result) {
    throw new Error("'sample_solution' textarea not found.");
  }
  if (solution === "") {
    solution_textarea.classList = "";
    sample_solution_result.innerHTML = "";
    return;
  }

  fetch('/api/validate_sample',
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(parse_solution(problem_id, solution))
    })
    .then(response => {
      if (!response.ok) {
        throw response.text();
      }
      return response.json();
    })
    .then(validation_result => {
      if (validation_result.Valid) {
        solution_textarea.classList = "success";
        sample_solution_result.innerHTML = validation_result.Valid;
      } else if (validation_result.Invalid) {
        solution_textarea.classList = "error";
        sample_solution_result.innerHTML = validation_result.Invalid;
      }
    })
    .catch(err => {
      solution_textarea.classList = "error";
      sample_solution_result.innerHTML = "Malformed solution, failed parsing.";
    });
}
