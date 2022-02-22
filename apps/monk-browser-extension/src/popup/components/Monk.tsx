import { useMonkContext } from "../context/MonkContext";

export function Monk() {
  const { pageInfo, monkUrl, error, uploadToMonk } = useMonkContext();

  if (error) {
    return <>Error: {error}</>;
  }

  return (
    <>
      <p className="font-mono">Add to Monk?</p>
      <pre>{JSON.stringify(pageInfo, null, 4)}</pre>
      <button
        className="border-2 border-black p-0.5 rounded"
        onClick={() => {
          uploadToMonk();
        }}
      >
        Upload to Monk
      </button>
    </>
  );
}
