let hasChanges = false;
const saveIndicator = document.getElementById("save-indicator");
let saveIndicatorTimeout = null;

function markDirty() {
    hasChanges = true;
}

function showSaved() {
    saveIndicator.classList.add("visible");

    clearTimeout(saveIndicatorTimeout);
    saveIndicatorTimeout = setTimeout(() => {
        saveIndicator.classList.remove("visible");
    }, 2000);
}

function collectContent() {
    const blocks = [];

    document.querySelectorAll(".grid-block").forEach(block => {
        let data = {
            col: block.dataset.col ?? "12",
            row: block.dataset.row ?? "1",
        }

        const ctx = new ComponentFetchDataContext(block, data);
        const comp = componentsRegistry.get(block.dataset.type);

        if (comp.hooks.fetchData) {
            data = {...data, ...comp.hooks.fetchData(ctx)};
        }

        blocks.push({
            name: block.dataset.type,
            data,
        });
    });

    return blocks;
}

function saveContent() {
    const new_content = collectContent();
    const body = JSON.stringify({
        post_id,
        new_content,
    });

    fetch('/api/posts', {
        method: 'PATCH',
        headers: {
            'Content-Type': 'application/json',
        },
        body,
    }).then(
        res => {
            if (res.ok) {
                console.log('changes saved');
                showSaved();
            } else {
                console.error(res);
            }
        },
        err => console.error(err)
    );
}

setInterval(() => {
    if (!hasChanges) return;

    saveContent();
    hasChanges = false;
}, 5000);