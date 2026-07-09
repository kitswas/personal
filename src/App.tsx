import ExpandableSankey from "./components/ExpandableSankey";

function App() {
	return (
		<div style={{ padding: "20px" }}>
			<h1>Personal - Expandable Sankey Demo</h1>
			<p>
				This is a working Solid + Unovis implementation of an expandable sankey
				diagram.
			</p>

			<div
				style={{
					"margin-top": "40px",
					border: "1px solid #ddd",
					padding: "20px",
					"border-radius": "8px",
				}}
			>
				<ExpandableSankey />
			</div>
		</div>
	);
}

export default App;
