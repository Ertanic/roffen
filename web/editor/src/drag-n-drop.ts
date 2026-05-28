import {markDirty} from "./save-content.ts";
import {canvas, componentsCache} from "./common.ts";
import {initProperties} from "./properties.ts";
import {renderHTML} from "./render.ts";

let dragged: HTMLElement | null = null;

type ClosestElement = {
    offset: number;
    element: HTMLElement | null;
};

function getDragAfterElement(container: HTMLDivElement, y: number) {
    const items: HTMLElement[] = [];
    container.querySelectorAll(".grid-block:not(.dragging)").forEach(el => items.push(el as HTMLElement));

    let closest: ClosestElement = {offset: Number.NEGATIVE_INFINITY, element: null};

    for (const el of items) {
        const box = el.getBoundingClientRect();
        const offset = y - (box.top + box.height / 2);

        if (offset < 0 && offset > closest.offset) {
            closest = {offset, element: el};
        }
    }

    return closest.element;
}

function initBlockDrag(block: HTMLDivElement) {
    block.draggable = true;

    block.addEventListener("dragstart", () => {
        dragged = block;
        block.classList.add("dragging");
    });

    block.addEventListener("dragend", () => {
        block.classList.remove("dragging");
        dragged = null;
        markDirty();
    });
}

document.querySelectorAll(".component").forEach(c => {
    const el = c as HTMLElement;
    c.addEventListener("dragstart", e => {
        const event = e as DragEvent;
        const type = el.dataset.type;

        if (!type) {
            console.error("no type found for component");
            return;
        }

        event.dataTransfer?.setData("type", type);
    });
});

export function initDrag() {
    if (!canvas) {
        console.error("no canvas found");
        return;
    }

    canvas.addEventListener("dragover", e => {
        e.preventDefault();

        const after = getDragAfterElement(canvas as HTMLDivElement, e.clientY);

        if (!dragged) return;

        if (after == null) {
            canvas?.appendChild(dragged);
        } else {
            canvas?.insertBefore(dragged, after);
        }
    });

    canvas.addEventListener("drop", e => {
        e.preventDefault();

        const event = e as DragEvent;

        const type = event.dataTransfer?.getData("type");

        if (type) {
            const block = document.createElement("div");
            block.classList.add("grid-block");
            block.dataset.type = type;

            const comp = componentsCache.get(type);

            if (!comp) {
                console.error("no component found for type: " + type);
                return;
            }

            const {html, state} = renderHTML(comp.html, comp.properties);
            block.appendChild(html);

            block.style.gridColumn = "span 12";

            block.dataset.col = "12";
            block.dataset.row = "1";

            initBlockDrag(block);
            initProperties(block, comp, state);

            canvas?.appendChild(block);

            markDirty();
        }
    });

    document.querySelectorAll(".grid-block").forEach(el => initBlockDrag(el as HTMLDivElement));
}