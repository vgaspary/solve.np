type SampleSolution =
  { TspSolution: number[] }
  | { GraphColoringSolution: number[] }
  | { VertexCoverSolution: number[] }
  | { MaximumCliqueSolution: number[] }
  | { MaximumCutSolution: number[] }
  | { SteinerTreeSolution: [number, number][] }
  | { KnapsackSolution: number[] }
  ;

function parseNumberList(input: string): number[] {
  const trimmedInput = input.trim();
  if (trimmedInput === '') {
    return [];
  }
  const numbers = trimmedInput.split(/\s+/).map(n => Number(n));
  if (numbers.some(isNaN)) {
    throw new Error("Malformed solution: All values must be valid numbers.");
  }
  return numbers;
}

export function parse_solution(problem_id: string, input: string): SampleSolution | null {
  switch (problem_id) {
    case 'travelling_salesman':
      return { TspSolution: parseNumberList(input) };
    case 'graph_coloring':
      return { GraphColoringSolution: parseNumberList(input) };
    case 'vertex_cover':
      return { VertexCoverSolution: parseNumberList(input) };
    case 'maximum_clique':
      return { MaximumCliqueSolution: parseNumberList(input) };
    case 'maximum_cut':
      return { MaximumCutSolution: parseNumberList(input) };
    case 'steiner_tree':
      return {
        SteinerTreeSolution: input.trim().split('\n').map(line => {
          const vertices = parseNumberList(line);
          if (vertices.length !== 2) {
            throw new Error("Malformed solution: Each line must contain exactly two numbers.");
          }
          return [vertices[0], vertices[1]] as [number, number];
        })
      };
    case 'knapsack':
      return { KnapsackSolution: parseNumberList(input) };
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

  (async () => {
    try {
      const parsed_solution_body = parse_solution(problem_id, solution);

      const response = await fetch('/api/validate_sample',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify(parsed_solution_body)
        });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || "Validation request failed");
      }

      const validation_result = await response.json();

      if (validation_result.Valid) {
        solution_textarea.classList = "success";
        sample_solution_result.innerHTML = validation_result.Valid;
      } else if (validation_result.Invalid) {
        solution_textarea.classList = "error";
        sample_solution_result.innerHTML = validation_result.Invalid;
      }
    } catch (err) {
      solution_textarea.classList = "error";
      if (err instanceof Error) {
        sample_solution_result.innerHTML = err.message;
      } else {
        sample_solution_result.innerHTML = "Malformed solution, failed parsing.";
      }
    }
  })();
}
