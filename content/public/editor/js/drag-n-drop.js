const canvas = document.getElementById("canvas");
const propertiesBody = document.getElementById("properties-body");

let dragged = null;

function getDragAfterElement(container, y) {
    const items = [...container.querySelectorAll(".grid-block:not(.dragging)")];

    let closest = {offset: Number.NEGATIVE_INFINITY, element: null};

    for (const el of items) {
        const box = el.getBoundingClientRect();
        const offset = y - (box.top + box.height / 2);

        if (offset < 0 && offset > closest.offset) {
            closest = {offset, element: el};
        }
    }

    return closest.element;
}

function initBlockDrag(block) {
    block.draggable = true;

    block.addEventListener("dragstart", () => {
        dragged = block;
        block.classList.add("dragging");
    });

    block.addEventListener("dragend", () => {
        block.classList.remove("dragging");
        dragged = null;
    });
}

document.querySelectorAll(".component").forEach(c => {
    c.addEventListener("dragstart", e => {
        e.dataTransfer.setData("type", c.dataset.type);
    });
});

canvas.addEventListener("dragover", e => {
    e.preventDefault();

    const after = getDragAfterElement(canvas, e.clientY);

    if (!dragged) return;

    if (after == null) {
        canvas.appendChild(dragged);
    } else {
        canvas.insertBefore(dragged, after);
    }
});

canvas.addEventListener("drop", e => {
    e.preventDefault();

    const type = e.dataTransfer.getData("type");

    if (type) {
        const block = document.createElement("div");
        block.classList.add("grid-block");
        block.dataset.type = type;

        const comp = componentsRegistry.get(type);
        const ctx = new ComponentMountContext(block, comp.html, comp.data, comp.title);

        if (comp.hooks.mount) {
            comp.hooks.mount(ctx);
        } else {
            const previewEl = document.createElement(comp.html);
            previewEl.innerText = comp.data.content ?? comp.title;
            block.appendChild(previewEl);
        }

        if (comp.hooks.setSize) {
            const ctx = new ComponentSetSizeContext(block, comp.data);
            comp.hooks.setSize(ctx);
        } else {
            block.style.gridColumn = "span 12";

            block.dataset.col = "12";
            block.dataset.row = "1";
        }

        initBlockDrag(block);
        initProperties(block, comp);

        canvas.appendChild(block);
    }
});

document.querySelectorAll(".grid-block").forEach(initBlockDrag);