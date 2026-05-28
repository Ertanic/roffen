import type {Component} from "common/src/Component.ts";
import {initDrag} from "./drag-n-drop.ts";
import {initDeleteButton} from "./delete-btn.ts";
import {componentsCache} from "./common.ts";
import {initProperties} from "./properties.ts";
import {markedDirty, saveContent, unmarkDirty} from "./save-content.ts";

fetch('/components').then((response: Response) => {
    if (!response.ok) {
        throw new Error("Failed to fetch components");
    }
    return response.json();
}).then((data: Component[]) => {
    for (const comp of data) {
        componentsCache.set(comp.name, comp);
    }

    initDrag();
    initDeleteButton();
    document.querySelectorAll('.grid-block').forEach(b => {
        const block = b as HTMLDivElement;
        const compName = block.dataset.type;
        if (!compName) return;
        const comp = componentsCache.get(compName);
        if (!comp) return;
        initProperties(block, comp, null);
    });

    setInterval(() => {
        if (!markedDirty()) return;

        saveContent();
        unmarkDirty();
    }, 5000);
});