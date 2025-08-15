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

function check_solution() {
  let soolution_textarea = document.getElementById("sample_solution");
  let solution = soolution_textarea.value;
  // TODO: request the server to validate the solution.
  if (solution === "") {
    soolution_textarea.classList = [""];
  } else if (Math.random() < 0.5) {
    soolution_textarea.classList = ["error"];
  } else {
    soolution_textarea.classList = ["success"];
  }
}
