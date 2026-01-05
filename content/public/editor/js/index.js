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

function initProperties(block) {
    block.addEventListener("click", () => {
        document.querySelectorAll(".grid-block.selected")
            .forEach(el => el.classList.remove("selected"));

        block.classList.add("selected");
        propertiesBody.innerHTML = "";

        /* CONTENT */
        const contentProp = createTextarea(
            "Content",
            block.dataset.content ?? block.innerText,
            value => block.children[0].innerText = value,
        );

        propertiesBody.appendChild(contentProp);

        /* GRID WIDTH */
        const colProp = createNumber(
            "Width (columns)",
            block.dataset.col ?? 12,
            1, 12,
            value => {
                block.dataset.col = value;
                block.style.gridColumn = `span ${value}`;
            }
        );

        propertiesBody.appendChild(colProp);

        /* GRID HEIGHT */
        const rowProp = createNumber(
            "Height (rows)",
            block.dataset.row ?? 1,
            1, 10,
            value => {
                block.dataset.row = value;
                block.style.gridRow = `span ${value}`;
            }
        );

        propertiesBody.appendChild(rowProp);

        /* TYPE-SPECIFIC */
        if (block.dataset.type === "title") {
            const levelProp = createNumber(
                "Heading level",
                block.dataset.level ?? 1,
                1, 6,
                value => {
                    block.innerHTML = `<h${value}>${block.children[0].innerText}</h${value}>`
                    block.dataset.level = value;
                }
            );

            propertiesBody.appendChild(levelProp);
        } else if (block.dataset.type === "image") {
            const srcProp = createUrlSource(
                "Image source",
                block.children[0].src,
                value => block.children[0].src = value,
            );
            propertiesBody.appendChild(srcProp);
        }
    });
}

function createUrlSource(label, value, onChange) {
    const wrap = document.createElement("div");
    wrap.className = "property";

    const l = document.createElement("label");
    l.textContent = label;

    const input = document.createElement("input");
    input.type = "url";
    input.value = value;

    input.addEventListener("input", e => onChange(e.target.value));

    wrap.append(l, input);
    return wrap;
}

function createTextarea(label, value, onChange) {
    const wrap = document.createElement("div");
    wrap.className = "property";

    const l = document.createElement("label");
    l.textContent = label;

    const ta = document.createElement("textarea");
    ta.value = value;

    ta.addEventListener("input", e => onChange(e.target.value));

    wrap.append(l, ta);
    return wrap;
}

function createNumber(label, value, min, max, onChange) {
    const wrap = document.createElement("div");
    wrap.className = "property";

    const l = document.createElement("label");
    l.textContent = label;

    const input = document.createElement("input");
    input.type = "number";
    input.min = min;
    input.max = max;
    input.value = value;

    input.addEventListener("input", e => onChange(+e.target.value));

    wrap.append(l, input);
    return wrap;
}


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

document.querySelectorAll(".grid-block").forEach(comp => {
    initBlockDrag(comp);
    initProperties(comp);
});