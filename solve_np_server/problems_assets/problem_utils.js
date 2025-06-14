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
