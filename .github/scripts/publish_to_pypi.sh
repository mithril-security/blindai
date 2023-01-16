# publish the python client to testpypi.

cd /blindaiv2/client

poetry config repositories.testpypi https://test.pypi.org/legacy/ # delete this line when moving from testpypi to pypi

poetry config pypi-token.testpypi $API_TOKEN_PYPI # replace testpypi by pypi

poetry publish --build --repository testpypi # remove --repository testpypi
