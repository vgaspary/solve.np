type SampleSolution = { TspSolution: number[] } | { GraphColoringSolution: number[] };

function parse_solution(problem_id: string, input: string): SampleSolution | null {
  if (problem_id === "travelling_salesman") {
    return { TspSolution: input.trim().split(/\s+/).map(n => Number(n)) };
  } else if (problem_id === "graph_coloring") {
    return { GraphColoringSolution: input.trim().split(/\s+/).map(n => Number(n)) };
  }
  return null;
}

function check_solution(problem_id: string): void {
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
    .then(response => response.json())
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
      solution_textarea.classList = "";
      sample_solution_result.innerHTML = "";
      console.log(err);
    });
}

function submit_solution(problem_id: string, solution_id: string, input_id: string): void {
  const solution_textarea = document.getElementById(input_id + '_solution') as HTMLTextAreaElement | null;
  if (!solution_textarea) {
    throw new Error(input_id + '_solution textarea not found.');
  }
  solution_textarea.classList = "";
  const solution = solution_textarea.value;
  if (solution.trim() === '') {
    return;
  }

  const status_paragraph = document.getElementById(input_id + '_status') as HTMLParagraphElement | null;
  if (!status_paragraph) {
    throw new Error(input_id + '_status paragraph not found.');
  }

  fetch('/api/submit_solution/' + problem_id + '/' + solution_id + '/' + input_id,
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
        status_paragraph.innerHTML = validation_result.Valid;
        solution_textarea.classList = "success";
      } else if (validation_result.Invalid) {
        status_paragraph.innerHTML = validation_result.Invalid;
        solution_textarea.classList = "error";
      }
    })
    .catch(err => {
      console.log(err);
    });

}
