export {};

declare global {
  interface Window {
    ai: {
      languageModel: {
        create(options?: AILanguageModelCreateOptions): Promise<AILanguageModel>;
        availability(options?: AILanguageModelAvailabilityOptions): Promise<AIAvailability>;
      };
    };
  }

  interface AILanguageModelAvailabilityOptions {
      expectedOutputs?: Array<{ type: "text", languages: string[] }>;
  }

  type AIAvailability = "readily" | "after-download" | "no";

  interface AILanguageModelCreateOptions {
    initialPrompts?: Array<{ role: "system" | "user" | "assistant", content: string }>;
    expectedOutputs?: Array<{ type: "text", languages: string[] }>;
  }

  interface AILanguageModel {
    prompt(input: string): Promise<string>;
    promptStreaming(input: string): ReadableStream<string>;
    destroy(): void;
    clone(): Promise<AILanguageModel>;
  }
}
