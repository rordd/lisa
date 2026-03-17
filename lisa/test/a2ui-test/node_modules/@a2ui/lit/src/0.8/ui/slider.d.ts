import { nothing } from "lit";
import { Root } from "./root.js";
import * as Primitives from "@a2ui/web_core/types/primitives";
import * as Types from "@a2ui/web_core/types/types";
export declare class Slider extends Root {
    #private;
    accessor value: Primitives.NumberValue | null;
    accessor minValue: number;
    accessor maxValue: number;
    accessor label: Primitives.StringValue | null;
    accessor inputType: Types.ResolvedTextField["type"] | null;
    static styles: import("lit").CSSResult[];
    render(): typeof nothing | import("lit-html").TemplateResult<1>;
}
//# sourceMappingURL=slider.d.ts.map