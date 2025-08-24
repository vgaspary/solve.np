function parse_solution(problem_id, input) {
    if (problem_id === "travelling_salesman") {
        return { TspSolution: input.trim().split(/\s+/).map(function (n) { return Number(n); }) };
    }
    else if (problem_id === "graph_coloring") {
        return { GraphColoringSolution: input.trim().split(/\s+/).map(function (n) { return Number(n); }) };
    }
    return null;
}
function check_solution(problem_id) {
    var solution_textarea = document.getElementById("sample_solution");
    if (!solution_textarea) {
        throw new Error("'sample_solution' textarea not found.");
    }
    var solution = solution_textarea.value;
    var sample_solution_result = document.getElementById('sample_solution_result');
    if (!sample_solution_result) {
        throw new Error("'sample_solution' textarea not found.");
    }
    if (solution === "") {
        solution_textarea.classList = "";
        sample_solution_result.innerHTML = "";
        return;
    }
    fetch('/api/validate_sample_solution', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(parse_solution(problem_id, solution))
    })
        .then(function (response) { return response.json(); })
        .then(function (validation_result) {
        if (validation_result.Valid) {
            solution_textarea.classList = "success";
            sample_solution_result.innerHTML = validation_result.Valid;
        }
        else if (validation_result.Invalid) {
            solution_textarea.classList = "error";
            sample_solution_result.innerHTML = validation_result.Invalid;
        }
    })
        .catch(function (err) {
        solution_textarea.classList = "";
        sample_solution_result.innerHTML = "";
        console.log(err);
    });
}
