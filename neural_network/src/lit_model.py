import hydra
import pytorch_lightning as pl
import torch
import torch.nn.functional as F
from omegaconf import DictConfig

from src.utils import get_logger

log = get_logger(__name__)


class LitModel(pl.LightningModule):
    def __init__(
        self,
        models_config: DictConfig,
        optimizer_config: DictConfig,
        scheduler_config: DictConfig,
        use_weights_path: str,
        board_size: int,
    ) -> None:
        super().__init__()

        self.save_hyperparameters()

        # Instantiate a model with random weights, or load them from a checkpoint
        if self.hparams.use_weights_path is None:
            for model_name, model_config in self.hparams.models_config.items():
                model = hydra.utils.instantiate(model_config)
                setattr(self, model_name, model)
        else:
            ckpt = LitModel.load_from_checkpoint(self.hparams.use_weights_path)

            for model_name in ckpt.hparams.models_config:
                model = getattr(ckpt, model_name)
                setattr(self, model_name, model)

        self.optimizer = hydra.utils.instantiate(self.hparams.optimizer_config, params=self.parameters())
        self.scheduler = (
            hydra.utils.instantiate(self.hparams.scheduler_config, optimizer=self.optimizer)
            if self.hparams.scheduler_config is not None
            else None
        )

        self.example_input_array = self.model.get_example_input_array()

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.model(x)

    def training_step(self, batch, _) -> torch.Tensor:
        states, policies, values = batch

        preds = self.model(states)
        policies_pred = preds[:, :-1]
        values_pred = preds[:, -1].unsqueeze(1)

        batch_size = policies.size(0)

        policy_loss = F.cross_entropy(policies_pred, policies)
        value_loss = F.mse_loss(values, values_pred)
        train_loss = policy_loss + value_loss

        self.log("policy_loss", policy_loss, prog_bar=True, logger=True)
        self.log("value_loss", value_loss, prog_bar=True, logger=True)
        self.log("train_loss", train_loss, prog_bar=True, logger=True)

        return train_loss

    def save_torchscript(self):
        self.eval()

        traced_script_module = torch.jit.trace(self.model, self.example_input_array.to(self.device)).to("cpu")
        traced_script_module.save("model.pt")

    def on_train_end(self):
        self.to_onnx("test.onnx")

        self.save_torchscript()

    def configure_optimizers(self):
        if self.scheduler is None:
            return self.optimizer

        return [self.optimizer], [{"scheduler": self.scheduler, "interval": "epoch"}]
