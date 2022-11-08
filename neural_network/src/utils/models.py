import torch
from torch import nn


class Model(nn.Module):

    def __init__(self) -> None:
        super().__init__()

        self.backbone = nn.Sequential(
            nn.Conv2d(in_channels=3, out_channels=64, kernel_size=5, padding=2),
            nn.SiLU(inplace=True),
            nn.Conv2d(in_channels=64, out_channels=128, kernel_size=5, padding=2),
            nn.SiLU(inplace=True),
        )
        self.flat_shape = self.get_flat_shape()

        self.policy = nn.Sequential(
            nn.Conv2d(in_channels=128, out_channels=64, kernel_size=5, padding=2),
            nn.SiLU(inplace=True),
            nn.Conv2d(in_channels=64, out_channels=1, kernel_size=5, padding=2),
        )

        self.value = nn.Sequential(
            nn.Linear(self.flat_shape, 128),
            nn.SiLU(inplace=True),
            nn.Linear(128, 1),
            nn.Tanh()
        )

    def forward(self, x: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        x = self.backbone(x)
        policy = self.policy(x)

        x = x.reshape(-1, self.flat_shape)
        value = self.value(x)

        return policy, value

    def get_flat_shape(self) -> int:
        x = torch.zeros(1, 3, 10, 10)
        out = self.backbone(x)
        return out.flatten().size(0)


def main():
    x = torch.randn(8, 3, 10, 10)
    model = Model()
    policy, value = model(x)
    print(policy.shape)
    print(value.shape)


if __name__ == "__main__":
    main()
