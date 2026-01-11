function normalizeComponents(block) {
    const type = block.dataset.type;
    const comp = componentsRegistry.get(type);

    if (comp.hooks.normalize) {
        const ctx = new ComponentNormalizeContext(block, comp.data);
        comp.hooks.normalize(ctx);
    }

    initProperties(block, comp);
}

document.querySelectorAll(".grid-block").forEach(normalizeComponents);