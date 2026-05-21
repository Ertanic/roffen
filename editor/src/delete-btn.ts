import {markDirty} from "./save-content.ts";

const grid = document.querySelector<HTMLElement>('.canvas-grid');
const deleteBtn = document.querySelector<HTMLButtonElement>('#component-remove');

let activeItem: HTMLElement | null = null;

export function initDeleteButton() {
    if (!grid || !deleteBtn) {
        console.warn("no grid or delete button elements found");
        return;
    }

    grid.addEventListener('mouseover', (e) => {
        const item = (e.target as HTMLElement).closest<HTMLElement>('.grid-block');
        if (!item) return;

        const rect = item.getBoundingClientRect();
        const parentRect = grid.getBoundingClientRect();

        deleteBtn.style.top = `${rect.top - parentRect.top - 16}px`;
        deleteBtn.style.left = `${rect.right - parentRect.left - 20}px`;

        deleteBtn.style.display = 'flex';
        deleteBtn.style.flexDirection = "column-reverse";
        activeItem = item;
    });

    grid.addEventListener('mouseleave', () => {
        deleteBtn.style.display = 'none';
        activeItem = null;
    });

    deleteBtn.addEventListener('click', () => {
        if (!activeItem) return;
        activeItem.remove();
        deleteBtn.style.display = 'none';
        markDirty();
    });
}