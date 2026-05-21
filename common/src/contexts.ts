type DataMap = Record<string, string>;

export interface IComponentMountContext {
}

export interface IComponentSetSizeContext {
    setRow(size: number): void;

    setCol(size: number): void;
}

export interface IComponentInitPropsContext {
    keepBlockInView(callback: () => void): void;

    autoResizeRows(): void;

    applyRows(rows: number): void;

    syncProperties(): void;

    createTextarea(label: string, defaultVal: string, onChange: (val: string) => void): void;

    createNumber(label: string, defaultVal: string, min: string, max: string, onChange: (val: string) => void): void;

    createUrlSource(label: string, value: string, onChange: (val: string) => void): void;

    get el(): HTMLElement;

    get data(): DataMap;

    get panel(): HTMLDivElement;

    get refs(): number;

    cancelAnimationFrame(n: number): void;

    requestAnimationFrame(callback: () => void): void;
}

export interface IComponentFetchDataContext {
    get el(): HTMLElement;

    get data(): DataMap;
}

export interface IComponentNormalizeContext {
    get el(): HTMLElement;

    get data(): DataMap;
}