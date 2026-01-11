class ComponentMountContext {
    constructor(el, html, data, title) {
        this.el = el;
        this.html = html;
        this.data = data;
        this.title = title;
    }
}

class ComponentSetSizeContext {
    constructor(el, data) {
        this.el = el;
        this.data = data;
    }

    setRow(size) {
        this.el.style.gridRow = `span ${size}`;
        this.el.dataset.row = size;
    }

    setCol(size) {
        this.el.style.gridColumn = `span ${size}`;
        this.el.dataset.col = size;
    }
}

class ComponentInitPropsContext {
    constructor(el, data, props, panel) {
        this.el = el;
        this.data = data;
        this.props = props;
        this.panel = panel;
        this.refs = [];
    }

    keepBlockInView(callback) {
        const canvas = document.querySelector(".editor-canvas");
        const before = this.el.getBoundingClientRect().top;

        callback();

        const after = this.el.getBoundingClientRect().top;
        canvas.scrollTop += (after - before);
    }

    autoResizeRows() {
        const rowHeight = 80;
        this.el.style.gridRow = "span 1";

        const contentHeight = this.el.children[0].scrollHeight;
        const rows = Math.max(1, Math.ceil(contentHeight / rowHeight));

        this.applyRows(rows);
    }

    applyRows(rows) {
        const value = Math.max(1, rows);

        this.el.dataset.row = String(value);
        this.el.style.gridRow = `span ${value}`;

        this.syncProperties();
        markDirty();
    }

    syncProperties() {
        if (!this.el.classList.contains("selected")) return;

        const rowInput = propertiesBody.querySelector('#height-prop');
        if (rowInput) {
            rowInput.value = Number(this.el.dataset.row);
        }
    }

    createTextarea(label, defaultVal, onChange) {
        const wrap = document.createElement("div");
        wrap.className = "property";

        const l = document.createElement("label");
        l.textContent = label;

        const ta = document.createElement("textarea");
        ta.value = defaultVal;

        ta.addEventListener("input", e => {
            onChange(e.target.value);
            markDirty();
        });

        wrap.append(l, ta);
        this.panel.appendChild(wrap);
        return wrap;
    }

    createNumber(label, defaultVal, min, max, onChange) {
        const wrap = document.createElement("div");
        wrap.className = "property";

        const l = document.createElement("label");
        l.textContent = label;

        const input = document.createElement("input");
        input.type = "number";
        input.min = min;
        input.max = max;
        input.value = defaultVal;

        input.addEventListener("input", e => {
            onChange(+e.target.value);
            markDirty();
        });

        wrap.append(l, input);
        this.panel.appendChild(wrap);
        return wrap;
    }

    createUrlSource(label, value, onChange) {
        const wrap = document.createElement("div");
        wrap.className = "property";

        const l = document.createElement("label");
        l.textContent = label;

        const input = document.createElement("input");
        input.type = "url";
        input.value = value;

        input.addEventListener("input", e => {
            onChange(e.target.value);
            markDirty();
        });

        wrap.append(l, input);
        this.panel.appendChild(wrap);
        return wrap;
    }
}

class ComponentFetchDataContext {
    constructor(el, data) {
        this.el = el;
        this.data = data;
    }
}