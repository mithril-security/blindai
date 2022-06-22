set -e
set -x

git config --global user.email "matthias.brunel@mithrilsecurity.io"
git config --global user.name "mbrunel"

git clone --single-branch --branch master "https://x-access-token:$API_TOKEN_GITHUB@github.com/mithril-security/gitbook_public.git" "./gitbook"

cp "docs/client.py.md" "gitbook/resources/client-api-reference/client-interface-$VERSION.md"
cd gitbook
python .github/scripts/update_summary.py $VERSION
git add .
git commit -m "Automatic update due to new released version of blindai client"
git push
