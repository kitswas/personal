import { VisSankey, VisSingleContainer, VisTooltip } from "@unovis/solid";
import { Sankey } from "@unovis/ts";
import { createEffect, createSignal } from "solid-js";

// Using zero-indexed indices for source/target just to guarantee D3 Sankey matches them properly
const data = {
	nodes: [
		{ id: "0", label: "Income" },
		{ id: "1", label: "Salary" },
		{ id: "2", label: "Investments" },
		{ id: "3", label: "Expenses" },
		{ id: "4", label: "Rent" },
		{ id: "5", label: "Food" },
		{ id: "6", label: "Savings" },
	],
	links: [
		{ source: 1, target: 0, value: 5000 },
		{ source: 2, target: 0, value: 1000 },
		{ source: 0, target: 3, value: 4000 },
		{ source: 0, target: 6, value: 2000 },
		{ source: 3, target: 4, value: 2500 },
		{ source: 3, target: 5, value: 1500 },
	],
};

export default function ExpandableSankey() {
	const [expandedNodes, setExpandedNodes] = createSignal<string[]>([]);

	// Simple logic to toggle node expansion
	const toggleNode = (node: any) => {
		setExpandedNodes((prev) =>
			prev.includes(node.id)
				? prev.filter((id) => id !== node.id)
				: [...prev, node.id],
		);
	};

	createEffect(() => {
		console.log("Expanded:", expandedNodes());
	});

	return (
		<div style="height: 400px; width: 100%; min-height: 400px; display: block;">
			<VisSingleContainer data={data} height={400}>
				<VisSankey
					nodePadding={20}
					nodeWidth={20}
					label={(n: any) => n.label}
					events={{
						[Sankey.selectors.node]: {
							click: toggleNode,
						},
					}}
				/>
				<VisTooltip />
			</VisSingleContainer>
		</div>
	);
}
