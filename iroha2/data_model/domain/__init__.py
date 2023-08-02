from typing import Union, Optional

from ...sys.iroha_data_model.domain import Domain as _Domain
from ...sys.iroha_data_model.domain import NewDomain as _NewDomain
from ...sys.iroha_data_model.domain import Id as _Id

from ..isi import Registrable

from .. import wrapper, patch


@wrapper(_Id)
class Id(_Id):

    @patch(_Id, "to_rust")
    def __repr__(self):
        return self.name


@wrapper(_Domain)
class Domain(_Domain, Registrable):

    def __init__(self, id: Union[Id, str], logo: Optional[str] = None):
        if isinstance(id, str):
            id = Id(id)

        super().__init__(id=id,
                         logo=logo,
                         accounts={},
                         asset_definitions={},
                         metadata={},)

    def registrable(self):
        return _NewDomain(id=self.id, logo=self.logo, metadata={})