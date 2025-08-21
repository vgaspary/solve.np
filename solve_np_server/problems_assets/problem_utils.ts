interface Problem {
  id: string;
  name: string;
  description: string;
}

type SampleSolution = { TspSolution: number[] } | { GraphColoringSolution: number[] };

function md_to_html(md_text: string): string {
  return md_text.replace(/(\*\*|\*|__|_)(.*?)(\*\*|\*|__|_)/g, '<em>$2</em>');
}

function render_problem(problem: Problem): string {
  return `
        <li>
          <h1><a href="/problems/${problem.id}">${problem.name}</a></h1>
          <article><p>${md_to_html(problem.description)}</p></article>
        </li>
          `;
}

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
