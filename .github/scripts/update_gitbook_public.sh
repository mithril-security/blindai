set -e
set -x

git clone --single-branch --branch master "https://x-access-token:$API_TOKEN_GITHUB@github.com/mithril-security/gitbook_public.git" "./gitbook"

cp "docs/client.py.md" "gitbook/resources/client-api-reference/client-interface-$VERSION.md"
cd gitbook
python .github/scripts/update_summary.py $VERSION
git add .
git commit --author="github-actions[bot] <41898282+github-actions[bot]@users.noreply.github.com>" -m "Automatic update due to new released version of blindai client"
git push
