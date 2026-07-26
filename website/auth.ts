import NextAuth from "next-auth";
import GitHub from "next-auth/providers/github";

export const { handlers, auth, signIn, signOut } = NextAuth({
  secret: process.env.AUTH_SECRET,
  trustHost: true,
  session: {
    strategy: "jwt",
  },
  pages: {
    signIn: "/login",
  },
  providers: [
    GitHub({
      clientId: process.env.GITHUB_ID,
      clientSecret: process.env.GITHUB_SECRET,
      profile(profile) {
        return {
          id: profile.id.toString(),
          name: profile.name ?? profile.login,
          email: profile.email,
          image: profile.avatar_url,
          githubUsername: profile.login,
        };
      },
    }),
  ],
  callbacks: {
    jwt({ token, user }) {
      if (user?.githubUsername) {
        token.githubUsername = user.githubUsername;
      }

      return token;
    },
    session({ session, token }) {
      if (typeof token.githubUsername === "string") {
        session.user.githubUsername = token.githubUsername;
      }

      return session;
    },
  },
});
