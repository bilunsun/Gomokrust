import torch
from torch import nn


class FlatModel(nn.Module):

    def __init__(self, size: int) -> None:
        super().__init__()

        self.size = size

        self.backbone = nn.Sequential(
            nn.Linear(size**2 + 1, 128),
            nn.SiLU(inplace=True),
            nn.Linear(128, 128),
            nn.SiLU(inplace=True),
        )

        self.policy_value = nn.Sequential(
            nn.Linear(128, 128),
            nn.SiLU(inplace=True),
            nn.Linear(128, size**2 + 1)
        )

    def forward(self, x: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        x = self.backbone(x)
        x = self.policy_value(x)
        return x

    def get_example_input_array(self) -> torch.Tensor:
        return torch.zeros(1, self.size**2 + 1)

class Model(nn.Module):

    def __init__(self) -> None:
        super().__init__()

        self.backbone = nn.Sequential(
            nn.Conv2d(in_channels=3, out_channels=64, kernel_size=3),
            nn.SiLU(inplace=True),
            nn.Conv2d(in_channels=64, out_channels=128, kernel_size=3),
            nn.SiLU(inplace=True),
        )
        self.flat_shape = self.get_flat_shape()
        print("self.flat_shape", self.flat_shape)

        self.policy = nn.Sequential(
            nn.Linear(self.flat_shape, 256),
            nn.SiLU(inplace=True),
            nn.Linear(256, 100)
        )

        self.value = nn.Sequential(
            nn.Linear(self.flat_shape, 128),
            nn.SiLU(inplace=True),
            nn.Linear(128, 1),
            nn.Tanh()
        )

    def forward(self, x: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        x = self.backbone(x)

        x = x.reshape(-1, self.flat_shape)

        policy = self.policy(x).reshape(-1, 1, 10, 10)
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
