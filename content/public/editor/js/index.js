const canvas = document.getElementById('canvas');
const properties = document.getElementById('properties');
const COLS = 12;
let selectedBlock = null;

/* ===== Drag from components ===== */

document.querySelectorAll('.component').forEach(c => {
    c.addEventListener('dragstart', e => {
        e.dataTransfer.setData('type', c.dataset.type);
    });
});

canvas.addEventListener('dragover', e => e.preventDefault());

canvas.addEventListener('drop', e => {
    e.preventDefault();
    const type = e.dataTransfer.getData('type');
    addBlock(type);
});

function addBlock(type) {
    const block = document.createElement('div');
    block.className = 'grid-block';
    block.dataset.type = type;

    block.style.gridColumn = 'span 12';
    block.style.gridRow = 'span 1';

    block.textContent = defaultText(type);

    addResizeHandle(block);
    block.onclick = () => selectBlock(block);

    canvas.appendChild(block);
}

function defaultText(type) {
    switch (type) {
        case 'title':
            return 'Post title';
        case 'text':
            return 'Text content...';
        case 'quote':
            return 'Quote...';
        case 'image':
            return 'Image URL';
        default:
            return '';
    }
}

/* ===== Selection ===== */

function selectBlock(block) {
    document.querySelectorAll('.grid-block').forEach(b => b.classList.remove('selected'));
    block.classList.add('selected');
    selectedBlock = block;
    renderProperties(block);
}

/* ===== Resize logic (grid-based) ===== */

function addResizeHandle(block) {
    const handle = document.createElement('div');
    handle.className = 'resize-handle';
    block.appendChild(handle);

    let startX, startSpan;

    handle.addEventListener('mousedown', e => {
        e.stopPropagation();
        startX = e.clientX;
        startSpan = getSpan(block);

        document.onmousemove = ev => resizeBlock(ev, block, startX, startSpan);
        document.onmouseup = stopResize;
    });
}

function getSpan(block) {
    return Number(block.style.gridColumn.replace('span ', '')) || 1;
}

function resizeBlock(e, block, startX, startSpan) {
    const delta = e.clientX - startX;
    const colWidth = canvas.clientWidth / COLS;
    const newSpan = Math.max(1, Math.min(COLS, startSpan + Math.round(delta / colWidth)));
    block.style.gridColumn = `span ${newSpan}`;
}

function stopResize() {
    document.onmousemove = null;
    document.onmouseup = null;
}

/* ===== Properties panel ===== */

function renderProperties(block) {
    const type = block.dataset.type;
    properties.innerHTML = `<h4>Properties</h4>`;

    properties.innerHTML += `
            <div class="property">
                <label>Grid span</label>
                <input class="input" type="number" min="1" max="${COLS}"
                    value="${getSpan(block)}">
            </div>
        `;

    properties.querySelector('input').oninput = e => {
        block.style.gridColumn = `span ${e.target.value}`;
    };

    if (type !== 'image') {
        properties.innerHTML += `
                <div class="property">
                    <label>Content</label>
                    <textarea class="input" rows="4">${block.textContent}</textarea>
                </div>
            `;
        properties.querySelector('textarea').oninput = e => {
            block.textContent = e.target.value;
        };
    }
}