import { describe, expect, it } from "vitest";
import { modalQuestSchema } from "../validation";

const paramsSchema = modalQuestSchema.pick(["params"]);

describe("tome parameter validation", () => {
    it("allows empty optional fields and requires other fields", async () => {
        const params = [
            { name: "name", value: "imix_svc" },
            { name: "executable_args", value: "", optional: true },
        ];
        await expect(paramsSchema.validate({ params })).resolves.toBeDefined();
        await expect(paramsSchema.validate({ params: [{ name: "name", value: "" }] })).rejects.toThrow("All parameters must have values");
    });
});
