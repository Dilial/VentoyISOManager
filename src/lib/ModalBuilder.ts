export interface ModalButton {
    label: string;
    action: (closeModal: () => void, inputValue?: string) => void | Promise<void>;
    primary?: boolean;
}

export interface ModalConfig {
    title: string;
    text: string;
    isHtml: boolean;
    hasInput: boolean;
    inputPlaceholder: string;
    buttons: ModalButton[];
}

export class ModalBuilder {
    private config: ModalConfig = {
        title: '',
        text: '',
        isHtml: false,
        hasInput: false,
        inputPlaceholder: '',
        buttons: []
    };

    setTitle(title: string): this {
        this.config.title = title;
        return this;
    }

    setText(text: string): this {
        this.config.text = text;
        this.config.isHtml = false;
        return this;
    }

    setHtmlText(html: string): this {
        this.config.text = html;
        this.config.isHtml = true;
        return this;
    }

    addInput(placeholder: string = ''): this {
        this.config.hasInput = true;
        this.config.inputPlaceholder = placeholder;
        return this;
    }

    addButton(label: string, action: (close: () => void, input?: string) => void | Promise<void>, primary: boolean = false): this {
        this.config.buttons.push({ label, action, primary });
        return this;
    }

    build(): ModalConfig {
        return this.config;
    }
}
