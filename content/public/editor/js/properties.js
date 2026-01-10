function initProperties(block, comp) {
    block.addEventListener("click", () => {
        document.querySelectorAll(".grid-block.selected")
            .forEach(el => el.classList.remove("selected"));

        block.classList.add("selected");
        propertiesBody.innerHTML = "";

        if (!comp) {
            comp = componentsRegistry.get(block.dataset.type);
        }

        const ctx = new ComponentInitPropsContext(block, comp.data, comp.props, propertiesBody);

        if (comp.hooks.initProps) {
            comp.hooks.initProps(ctx);
        }

        /* GRID WIDTH */
        ctx.createNumber(
            "Width (columns)",
            block.dataset.col ?? 12,
            1, 12,
            value => {
                block.dataset.col = value;
                block.style.gridColumn = `span ${value}`;
            }
        );

        /* GRID HEIGHT */
        ctx.createNumber(
            "Height (rows)",
            block.dataset.row ?? 1,
            1, "",
            value => ctx.applyRows(value)
        );
    });
}

document.querySelectorAll(".grid-block").forEach(initProperties);