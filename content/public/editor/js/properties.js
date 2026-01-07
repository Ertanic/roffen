function keepBlockInView(block, fn) {
    const canvas = document.querySelector(".editor-canvas");
    const before = block.getBoundingClientRect().top;

    fn();

    const after = block.getBoundingClientRect().top;
    canvas.scrollTop += (after - before);
}

function autoResizeRows(block) {
    const rowHeight = 80;
    block.style.gridRow = "span 1";

    const contentHeight = block.children[0].scrollHeight;
    const rows = Math.max(1, Math.ceil(contentHeight / rowHeight));

    applyRows(block, rows);
}

function applyRows(block, rows) {
    const value = Math.max(1, rows);

    block.dataset.row = String(value);
    block.style.gridRow = `span ${value}`;

    syncProperties(block);
}

function syncProperties(block) {
    if (!block.classList.contains("selected")) return;

    const rowInput = propertiesBody.querySelector('#height-prop');
    if (rowInput) {
        rowInput.value = Number(block.dataset.row);
    }
}

function initProperties(block) {
    block.addEventListener("click", () => {
        document.querySelectorAll(".grid-block.selected")
            .forEach(el => el.classList.remove("selected"));

        block.classList.add("selected");
        propertiesBody.innerHTML = "";

        let raf;

        /* CONTENT */
        const contentProp = createTextarea(
            "Content",
            block.dataset.content ?? block.innerText,
            value => {
                cancelAnimationFrame(raf);
                raf = requestAnimationFrame(() => {
                    keepBlockInView(block, () => {
                        block.children[0].innerText = value;
                        autoResizeRows(block);
                    });
                });
            },
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
            1, "",
            value => applyRows(block, value),
            "height-prop"
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

function createNumber(label, value, min, max, onChange, id = null) {
    const wrap = document.createElement("div");
    wrap.className = "property";

    const l = document.createElement("label");
    l.textContent = label;

    const input = document.createElement("input");
    input.type = "number";
    input.min = min;
    input.max = max;
    input.value = value;

    if (id) {
        input.id = id;
    }

    input.addEventListener("input", e => onChange(+e.target.value));

    wrap.append(l, input);
    return wrap;
}

document.querySelectorAll(".grid-block").forEach(initProperties);