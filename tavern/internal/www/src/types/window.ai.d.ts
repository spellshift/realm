export {};

declare global {
  interface Window {
    ai: {
      languageModel: {
        create(options?: AILanguageModelCreateOptions): Promise<AILanguageModel>;
        capabilities(): Promise<AILanguageModelCapabilities>;
      };
    };
  }

  interface AILanguageModelCreateOptions {
    systemPrompt?: string;
  }

  interface AILanguageModel {
    prompt(input: string): Promise<string>;
    promptStreaming(input: string): ReadableStream<string>;
    destroy(): void;
    clone(): Promise<AILanguageModel>;
  }

  interface AILanguageModelCapabilities {
    available: "readily" | "after-download" | "no";
    defaultTemperature: number;
    defaultTopK: number;
    maxTopK: number;
  }
}
