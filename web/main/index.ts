import 'highlight.js/styles/github.css';
import hljs from 'highlight.js';
import rust from 'highlight.js/lib/languages/rust';
import typescript from 'highlight.js/lib/languages/typescript';
import markdown from 'highlight.js/lib/languages/markdown';
import powershell from 'highlight.js/lib/languages/powershell';
import toml from 'highlight.js/lib/languages/ini';
import handlebars from 'highlight.js/lib/languages/handlebars';
import mermaid from 'mermaid';

hljs.registerLanguage('rust', rust);
hljs.registerLanguage('typescript', typescript);
hljs.registerLanguage('html', markdown);
hljs.registerLanguage('handlebars', handlebars);
hljs.registerLanguage('powershell', powershell);
hljs.registerLanguage('toml', toml);

hljs.highlightAll();

const content = document.getElementById('content');

type ContentTableItem = {
    level: number;
    header: HTMLElement;
    li: HTMLLIElement;
}

let last: ContentTableItem | null = null;
content?.querySelectorAll('h1,h2,h3,h4,h5,h6').forEach(h => {
    const header = h as HTMLElement;

    const id = header.textContent.replaceAll(' ', '-');
    header.id = id;

    const link = `#${id}`;
    const list = document.getElementById('contents-table-list');
    const li = document.createElement('li');
    const a = document.createElement('a');

    a.href = link;
    a.textContent = header.textContent;
    li.appendChild(a);

    if (!last) {
        list?.appendChild(li);
        const level = Number(header.tagName.replace('H', ''));
        last = {level, header, li};
        return;
    }

    const level = Number(header.tagName.replace('H', ''));

    if (level > last.level) {
        const ol = document.createElement('ol');
        ol.appendChild(li);
        last.li.appendChild(ol);
        last = {level, header, li};
        return;
    } else if (level < last.level) {
        last.li.parentElement?.parentElement?.appendChild(li);
    } else {
        last.li.parentElement?.appendChild(li);
    }

    last = {level, header, li};
});

mermaid.initialize({
    startOnLoad: true,
});