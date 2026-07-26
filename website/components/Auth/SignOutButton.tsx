import { LogOut } from "lucide-react";
import { signOut } from "../../auth";

export function SignOutButton() {
  return (
    <form
      action={async () => {
        "use server";
        await signOut({ redirectTo: "/" });
      }}
    >
      <button
        type="submit"
        className="inline-flex min-h-11 items-center justify-center gap-2 rounded-[12px] border-2 border-ink bg-yellow px-4 text-sm font-black shadow-brutal-sm transition-transform hover:-translate-y-0.5 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-orange/35"
      >
        <LogOut aria-hidden="true" className="size-4" strokeWidth={2.7} />
        Sign out
      </button>
    </form>
  );
}
