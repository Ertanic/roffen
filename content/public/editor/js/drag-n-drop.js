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
        let previewElement = null;
        switch (type) {
            case "title":
                previewElement = document.createElement("h1");
                previewElement.textContent = "Title";
                break;
            case "text":
                previewElement = document.createElement("p");
                previewElement.textContent = "Text";
                break;
            case "image":
                previewElement = document.createElement("img");
                previewElement.src = "/editor/img/image-placeholder.png";
                break;
            case "quote":
                previewElement = document.createElement("blockquote");
                previewElement.textContent = "Quote";
                break;
        }

        const block = document.createElement("div");

        block.classList.add("grid-block");

        if (type === "image") {
            block.classList.add("component-image");
            block.style.gridRow = "span 4";
            block.style.gridColumn = "span 6";

            block.dataset.col = "6";
            block.dataset.row = "4";
        } else {
            block.style.gridColumn = "span 12";

            block.dataset.col = "12";
            block.dataset.row = "1";
        }

        block.dataset.type = type;

        initBlockDrag(block);
        initProperties(block);

        block.appendChild(previewElement);
        canvas.appendChild(block);
    }
});

document.querySelectorAll(".grid-block").forEach(initBlockDrag);