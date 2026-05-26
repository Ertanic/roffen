import {ComponentFetchDataContext} from "./component-contexts.ts";
import {componentsRegistry, get_query} from "./common.ts";

let hasChanges = false;
const saveIndicator = document.getElementById("save-indicator");
let saveIndicatorTimeout: NodeJS.Timeout | null = null;

type Block = {
    name: string;
    data: Record<string, string>;
}

export function markDirty() {
    hasChanges = true;
}

export function markedDirty(): boolean {
    return hasChanges;
}

export function unmarkDirty() {
    hasChanges = false;
}

function showSaved() {
    saveIndicator?.classList.add("visible");

    if (saveIndicatorTimeout) {
        clearTimeout(saveIndicatorTimeout);
        return;
    }

    saveIndicatorTimeout = setTimeout(() => {
        saveIndicator?.classList.remove("visible");
    }, 2000);
}

function collectContent() {
    const blocks: Block[] = [];

    document.querySelectorAll(".grid-block").forEach((el: Element) => {
        const block = el as HTMLDivElement;

        let data: Record<string, string> = {
            col: block.dataset.col ?? "12",
            row: block.dataset.row ?? "1",
        }

        const type = block.dataset.type;
        if (!type) return;

        const ctx = new ComponentFetchDataContext(block, data);
        const comp = componentsRegistry.get(type);

        if (comp?.hooks.fetchData) {
            data = {...data, ...comp.hooks.fetchData(ctx)};
        }

        blocks.push({
            name: type,
            data,
        });
    });

    return blocks;
}

export function saveContent() {
    const query = get_query();
    const post_id = query["id"];

    if (!post_id) {
        console.error("No post_id found in query");
        return;
    }

    const new_content = collectContent();
    const body = JSON.stringify({
        post_id,
        new_content,
    });

    fetch('/api/posts', {
        method: 'PATCH',
        headers: {
            'Content-Type': 'application/json',
        },
        body,
    }).then(
        res => {
            if (res.ok) {
                console.log('changes saved');
                showSaved();
            } else {
                console.error(res);
            }
        },
        err => console.error(err)
    );
}