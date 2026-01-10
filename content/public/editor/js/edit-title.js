const titleText = document.getElementById("article-title-text");
const editBtn = document.getElementById("title-edit-btn");

editBtn.addEventListener("click", () => {
    const current = titleText.innerText;

    const next = prompt("Edit article title", current);

    if (next === null) return;

    const normalized = next.trim();
    if (!normalized) return;

    titleText.innerText = normalized;

    const body = {
        post_id: post_id,
        new_title: normalized,
    };

    fetch('/api/posts', {
        method: 'PATCH',
        body: JSON.stringify(body),
    }).then(
        () => console.log('new title saved'),
        err => console.error(err)
    );
});