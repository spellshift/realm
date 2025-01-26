/* eslint-disable react/jsx-props-no-spreading */
import { FC, useMemo } from "react";
import { solidBadge } from "./BadgeStyles";
import { VariantProps } from "tailwind-variants";

// extend the base button attributes
interface BadgeProps {
    leftIcon?: React.ReactElement;
    rightIcon?: React.ReactElement;
    className?: string,
    children: any,
    badgeStyle?: VariantProps<typeof solidBadge>;
}

const Badge: FC<BadgeProps> = (
    { children, leftIcon, rightIcon, badgeStyle, className, ...rest }
) => {

    // determine icon placement
    const { newIcon: icon, iconPlacement } = useMemo(() => {
        let newIcon = rightIcon || leftIcon;

        return {
            newIcon,
            iconPlacement: rightIcon ? ("right" as const) : ("left" as const),
        };
    }, [leftIcon, rightIcon]);

    const renderBadgeVariant = () => {
        return solidBadge({ ...badgeStyle, className })
    }

    return (
        <div className="flex flex-row">
            <div
                className={renderBadgeVariant()}
                {...rest}
            >
                {/** render icon before */}
                {icon && iconPlacement === "left" ? (
                    <span className={`inline-flex shrink-0 self-center ${children}`}>{icon}</span>
                ) : null}

                {children}

                {/** render icon after */}
                {icon && iconPlacement === "right" ? (
                    <span className={`inline-flex shrink-0 self-center  ${children}`}>{icon}</span>
                ) : null}
            </div>
        </div>
    );
};

// set default props
Badge.defaultProps = {
    leftIcon: undefined,
    rightIcon: undefined,
    badgeStyle: {
        color: "gray"
    }
};

export default Badge;
