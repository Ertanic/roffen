const grid = document.querySelector('.canvas-grid');
const deleteBtn = document.querySelector('#component-remove');

console.log(deleteBtn)

let activeItem = null;

grid.addEventListener('mouseover', (e) => {
    const item = e.target.closest('.grid-block');
    if (!item) return;

    console.log(item)

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
});